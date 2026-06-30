use crate::arg_parser::{FixMode, FixRule};
use crate::formatter::checker::is_buf_utf8;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::path::PathBuf;

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
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if !content.starts_with(BOM) {
        return Ok(false);
    }

    // Rewrite the file in place: shift the body to the front and drop the now
    // stale trailing bytes. No temp file / rename, so there is no chance of the
    // Windows "replace a file with an open handle" error (os error 5).
    let body = &content[BOM.len()..];
    file.seek(SeekFrom::Start(0))?;
    file.write_all(body)?;
    file.set_len(body.len() as u64)?;
    println!("Removed BOM from {}", path.display());
    Ok(true)
}

/// add utf-8 BOM mark to given file if the file is utf-8 encoded
fn add_bom(path: &PathBuf) -> Result<bool, Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if content.starts_with(BOM) {
        return Ok(false);
    }
    if !is_buf_utf8(&content) {
        return Ok(false);
    }

    // Rewrite the file in place. The file only grows (by BOM.len()), so the old
    // content is fully overwritten and no truncation is needed.
    file.seek(SeekFrom::Start(0))?;
    file.write_all(BOM)?;
    file.write_all(&content)?;
    println!("Added BOM to {}", path.display());
    Ok(true)
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

    /// Regression test for issue #5 (Windows "os error 5"). add_bom/remove_bom
    /// must succeed end-to-end and round-trip cleanly. With the in-place
    /// implementation there is no rename at all, so the original failure mode
    /// cannot occur; this still guards the behavior on all platforms.
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

    #[test]
    fn add_bom_on_empty_file_yields_only_bom() {
        let dir = tempdir().unwrap();
        let path = write_file(dir.path(), "empty.txt", b"");

        let changed = add_bom(&path).unwrap();

        assert!(changed);
        assert_eq!(fs::read(&path).unwrap(), BOM);
    }
}
