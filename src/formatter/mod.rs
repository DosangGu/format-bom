mod bom_formatter;
mod checker;

use crate::arg_parser::FixRule;
use bom_formatter::BomFormatter;
use std::{error::Error, path::PathBuf};

pub fn format_bom(files: &[PathBuf], fix_rule: &FixRule) -> Result<(), Box<dyn Error>> {
    let mut formatter = BomFormatter::new(fix_rule);
    formatter.register_files(files);

    formatter.format()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::format_bom;
    use crate::arg_parser::{FixMode, FixRule};
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    const BOM: &[u8] = b"\xEF\xBB\xBF";

    #[test]
    fn format_bom_adds_and_removes_in_one_pass() {
        let dir = tempdir().unwrap();

        // No BOM, not in any extension set -> follows Add mode -> gets a BOM.
        let add_path = dir.path().join("a.txt");
        fs::write(&add_path, b"hello").unwrap();

        // Has a BOM, extension is in ext_remove -> BOM is removed.
        let remove_path = dir.path().join("b.md");
        let mut with_bom = BOM.to_vec();
        with_bom.extend_from_slice(b"world");
        fs::write(&remove_path, &with_bom).unwrap();

        let rule = FixRule {
            mode: FixMode::Add,
            ext_add: HashSet::new(),
            ext_remove: ["md"].iter().map(|e| e.to_string()).collect(),
        };
        let files: Vec<PathBuf> = vec![add_path.clone(), remove_path.clone()];

        format_bom(&files, &rule).unwrap();

        let mut expected_added = BOM.to_vec();
        expected_added.extend_from_slice(b"hello");
        assert_eq!(fs::read(&add_path).unwrap(), expected_added);
        assert_eq!(fs::read(&remove_path).unwrap(), b"world");
    }
}
