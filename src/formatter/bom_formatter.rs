use crate::arg_parser::{FixMode, FixRule};
use crate::formatter::checker::is_buf_utf8;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::path::PathBuf;
use tempfile::NamedTempFile;

const BOM: &[u8] = b"\xEF\xBB\xBF";

pub struct BomFormatter<'a> {
    fix_rule: &'a FixRule,
    files_to_add_bom: Vec<&'a PathBuf>,
    files_to_remove_bom: Vec<&'a PathBuf>,
}

impl<'a> BomFormatter<'a> {
    pub fn new(fix_rule: &'a FixRule) -> Self {
        Self {
            fix_rule,
            files_to_add_bom: Vec::new(),
            files_to_remove_bom: Vec::new(),
        }
    }

    pub fn register_files(&mut self, files: &'a [PathBuf]) {
        self.register_add_bom(files);
        self.register_remove_bom(files);

        let files_etc: Vec<&PathBuf> = files
            .iter()
            .filter(|file| {
                !self.fix_rule.ext_add.contains(&get_extension(file))
                    && !self.fix_rule.ext_remove.contains(&get_extension(file))
            })
            .collect();

        match self.fix_rule.mode {
            FixMode::Add => self.files_to_add_bom.extend(files_etc),
            FixMode::Remove => self.files_to_remove_bom.extend(files_etc),
        }
    }

    pub fn format(&self) -> Result<(), Box<dyn Error>> {
        for file in &self.files_to_add_bom {
            if let Err(err) = add_bom(file) {
                println!("adding bom failed: {}", err);
            }
        }

        for file in &self.files_to_remove_bom {
            if let Err(err) = remove_bom(file) {
                println!("removing bom failed: {}", err);
            }
        }

        Ok(())
    }

    fn register_add_bom(&mut self, files: &'a [PathBuf]) {
        let files_to_add_bom: Vec<&PathBuf> = files
            .iter()
            .filter(|&file| self.fix_rule.ext_add.contains(&get_extension(file)))
            .collect();
        self.files_to_add_bom.extend(files_to_add_bom);
    }

    fn register_remove_bom(&mut self, files: &'a [PathBuf]) {
        let files_to_remove_bom: Vec<&PathBuf> = files
            .iter()
            .filter(|&file| self.fix_rule.ext_remove.contains(&get_extension(file)))
            .collect();
        self.files_to_remove_bom.extend(files_to_remove_bom);
    }
}

fn get_extension(path: &Path) -> String {
    path.extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string()
}

/// remove utf-8 BOM mark of given file
fn remove_bom(path: &PathBuf) -> Result<bool, Box<dyn Error>> {
    println!("Removing BOM from {}", path.display());
    let mut reader = get_file_reader(path)?;

    let mut buf = vec![0; BOM.len()];
    reader.read_exact(&mut buf)?;

    if buf != BOM {
        return Ok(false);
    }

    let mut temp_file = NamedTempFile::new_in(path.parent().unwrap())?;
    {
        let mut writer = BufWriter::new(&mut temp_file);
        io::copy(&mut reader, &mut writer)?;
    }
    // Close the handle to the original file before persisting; on Windows a
    // rename cannot replace a file that still has an open handle (os error 5).
    drop(reader);
    temp_file.persist(path)?;
    println!("Removed BOM from {}", path.display());
    Ok(true)
}

/// add utf-8 BOM mark to given file if the file is utf-8 encoded
fn add_bom(path: &PathBuf) -> Result<bool, Box<dyn Error>> {
    let mut reader = get_file_reader(path)?;

    let mut buf = vec![0; BOM.len()];

    reader.read_exact(&mut buf)?;

    if buf == BOM {
        return Ok(false);
    }

    reader.read_to_end(&mut buf)?;
    if !is_buf_utf8(&buf) {
        return Ok(false);
    }

    // Close the handle to the original file before persisting; on Windows a
    // rename cannot replace a file that still has an open handle (os error 5).
    drop(reader);

    let mut temp_file = NamedTempFile::new_in(path.parent().unwrap())?;
    {
        let mut writer = BufWriter::new(&mut temp_file);
        writer.write_all(BOM)?;
        writer.write_all(&buf)?;
    }
    temp_file.persist(path)?;
    println!("Added BOM to {}", path.display());
    Ok(true)
}

fn get_file_reader(path: &Path) -> Result<BufReader<File>, Box<dyn Error>> {
    File::open(path).map(BufReader::new).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn add_bom_adds_bom_to_utf8_file_without_bom() {
        let dir = tempdir().unwrap();
        let path = write_file(dir.path(), "a.txt", b"hello world");

        let changed = add_bom(&path).unwrap();

        assert!(changed);
        let content = fs::read(&path).unwrap();
        assert_eq!(&content[..3], BOM);
        assert_eq!(&content[3..], b"hello world");
    }

    #[test]
    fn add_bom_is_noop_when_bom_already_present() {
        let dir = tempdir().unwrap();
        let mut bytes = BOM.to_vec();
        bytes.extend_from_slice(b"hello");
        let path = write_file(dir.path(), "a.txt", &bytes);

        let changed = add_bom(&path).unwrap();

        assert!(!changed);
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn add_bom_skips_non_utf8_file() {
        let dir = tempdir().unwrap();
        // Not starting with a BOM and not valid UTF-8.
        let bytes = [0x00u8, 0xff, 0xfe, 0x41, 0x42];
        let path = write_file(dir.path(), "a.bin", &bytes);

        let changed = add_bom(&path).unwrap();

        assert!(!changed);
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn remove_bom_removes_existing_bom() {
        let dir = tempdir().unwrap();
        let mut bytes = BOM.to_vec();
        bytes.extend_from_slice(b"hello world");
        let path = write_file(dir.path(), "a.txt", &bytes);

        let changed = remove_bom(&path).unwrap();

        assert!(changed);
        assert_eq!(fs::read(&path).unwrap(), b"hello world");
    }

    #[test]
    fn remove_bom_is_noop_when_no_bom() {
        let dir = tempdir().unwrap();
        let path = write_file(dir.path(), "a.txt", b"hello world");

        let changed = remove_bom(&path).unwrap();

        assert!(!changed);
        assert_eq!(fs::read(&path).unwrap(), b"hello world");
    }

    /// Regression test for issue #5: on Windows, persisting the temp file over a
    /// path that still had an open read handle failed with "os error 5" (access
    /// denied). add_bom/remove_bom must now succeed end-to-end. This runs on all
    /// platforms and specifically guards the Windows code path.
    #[test]
    fn add_then_remove_round_trip_succeeds() {
        let dir = tempdir().unwrap();
        let original = b"Write-Host \"hello\"\n";
        let path = write_file(dir.path(), "install_pfx.ps1", original);

        assert!(add_bom(&path).unwrap());
        let with_bom = fs::read(&path).unwrap();
        assert_eq!(&with_bom[..3], BOM);
        assert_eq!(&with_bom[3..], original);

        assert!(remove_bom(&path).unwrap());
        assert_eq!(fs::read(&path).unwrap(), original);
    }
}
