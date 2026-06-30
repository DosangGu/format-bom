use ignore::WalkBuilder;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Get all files in a directory.
pub fn get_file_list(path: &PathBuf) -> Vec<PathBuf> {
    let walker = WalkBuilder::new(path).build();
    let mut file_list: Vec<PathBuf> = Vec::new();
    for entry in walker.flatten() {
        if entry.file_type().unwrap().is_file() {
            file_list.push(entry.into_path());
        }
    }
    file_list
}

#[allow(unused)]
/// Filter files by gitignore.
fn filter_by_gitignore(file_list: Vec<PathBuf>) -> Vec<PathBuf> {
    let gitignore_pattern = generate_gitignore_regex_patterns(&PathBuf::from(".gitignore"));
    file_list
        .into_iter()
        .filter(|file| !is_ignored_by_gitignore(file, &gitignore_pattern))
        .collect()
}

/// Check if a file is ignored by gitignore.
fn is_ignored_by_gitignore(file: &Path, gitignore_pattern: &[Regex]) -> bool {
    for re in gitignore_pattern {
        if re.is_match(file.to_str().unwrap()) {
            return true;
        }
    }
    false
}

fn generate_gitignore_regex_patterns(gitignore_file: &PathBuf) -> Vec<Regex> {
    let mut patterns: Vec<Regex> = Vec::new();
    if !gitignore_file.exists() {
        return patterns;
    }
    let gitignore = std::fs::read_to_string(gitignore_file).unwrap();
    let gitignore = gitignore.split('\n');
    for line in gitignore {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let line = line.replace('.', "\\.");
        let line = line.replace('*', ".*");
        let line = line.replace('?', ".");
        let line = format!("^{}$", line);
        let re = Regex::new(&line).unwrap();
        patterns.push(re);
    }
    patterns
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs;
    use tempfile::tempdir;

    fn listed_set(path: &Path) -> HashSet<PathBuf> {
        get_file_list(&path.to_path_buf()).into_iter().collect()
    }

    #[test]
    fn get_file_list_returns_files_in_dir_and_subdirs() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.txt");
        fs::write(&a, b"a").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        let b = dir.path().join("sub").join("b.txt");
        fs::write(&b, b"b").unwrap();

        let found = listed_set(dir.path());

        assert_eq!(found, HashSet::from([a, b]));
    }

    #[test]
    fn get_file_list_skips_hidden_files() {
        let dir = tempdir().unwrap();
        let visible = dir.path().join("visible.txt");
        fs::write(&visible, b"v").unwrap();
        fs::write(dir.path().join(".hidden"), b"h").unwrap();

        let found = listed_set(dir.path());

        assert_eq!(found, HashSet::from([visible]));
    }
}
