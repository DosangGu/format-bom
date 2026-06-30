mod fix_mode;
mod fix_mode_args;
mod fix_rule;

pub use fix_mode::FixMode;
use fix_mode_args::FixModeArgs;
pub use fix_rule::FixRule;
use std::{collections::HashSet, error::Error};

use crate::Args;

pub fn parse_args(argument: &Args) -> Result<FixRule, Box<dyn std::error::Error>> {
    get_fix_mode(argument).and_then(|fix_mode_args| get_fix_rule(argument, &fix_mode_args))
}

fn get_fix_mode(fix_option: &Args) -> Result<FixModeArgs, Box<dyn std::error::Error>> {
    let mode: [bool; 3] = [fix_option.add, fix_option.remove, fix_option.add_strict];

    if mode.into_iter().filter(|&x| x).count() > 1 {
        Err("You can not set multiple fix mode.".into())
    } else if fix_option.add {
        Ok(FixModeArgs::Add)
    } else if fix_option.remove {
        Ok(FixModeArgs::Remove)
    } else if fix_option.add_strict {
        Ok(FixModeArgs::AddStrict)
    } else {
        Ok(FixModeArgs::Remove)
    }
}

fn get_fix_rule(argument: &Args, fix_mode_args: &FixModeArgs) -> Result<FixRule, Box<dyn Error>> {
    let mut fix_rule = FixRule {
        mode: FixMode::Remove,
        ext_add: HashSet::new(),
        ext_remove: HashSet::new(),
    };

    let result_set = match fix_mode_args {
        FixModeArgs::Add => fix_rule.set_fix_rule_for_add(argument),
        FixModeArgs::Remove => fix_rule.set_fix_rule_for_remove(argument),
        FixModeArgs::AddStrict => fix_rule.set_fix_rule_for_add_strict(argument),
    };

    match result_set {
        Ok(_) => Ok(fix_rule),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_mode() {
        let args = Args {
            path: None,
            add: false,
            remove: false,
            add_strict: false,
            add_bom: None,
            remove_bom: None,
        };

        let fix_rule = parse_args(&args).unwrap();

        assert_eq!(fix_rule.mode, FixMode::Remove);
    }

    #[test]
    fn test_set_mode_as_add() {
        let args = Args {
            path: None,
            add: true,
            remove: false,
            add_strict: false,
            add_bom: None,
            remove_bom: None,
        };
        let expected_remove = vec![
            "html", "css", "svg", // web development
            "js", "ts", // programming language
            "md", // document
            "json", "toml", "yaml", "csv", "xml", // data
            "ini", "conf", "cfg", // config
            "sh", "bat", "ps1", // other
        ]
        .iter()
        .map(|x| x.to_string())
        .collect::<HashSet<String>>();

        let fix_rule = parse_args(&args).unwrap();

        assert_eq!(fix_rule.mode, FixMode::Add);
        assert_eq!(fix_rule.ext_remove, expected_remove);
    }

    #[test]
    fn test_set_mode_as_remove() {
        let args = Args {
            path: None,
            add: false,
            remove: true,
            add_strict: false,
            add_bom: None,
            remove_bom: None,
        };

        let fix_rule = parse_args(&args).unwrap();

        assert_eq!(fix_rule.mode, FixMode::Remove);
    }

    #[test]
    fn test_set_mode_as_add_strict() {
        let args = Args {
            path: None,
            add: false,
            remove: false,
            add_strict: true,
            add_bom: None,
            remove_bom: None,
        };

        let fix_rule = parse_args(&args).unwrap();

        assert_eq!(fix_rule.mode, FixMode::Add);
        assert_eq!(fix_rule.ext_remove, HashSet::new());
    }

    #[test]
    fn test_set_mode_as_add_with_add_bom() {
        let args = Args {
            path: None,
            add: true,
            remove: false,
            add_strict: false,
            add_bom: Some(vec!["html".to_string()]),
            remove_bom: None,
        };

        let fix_rule_result = parse_args(&args);
        assert!(fix_rule_result.is_err());
        assert_eq!(
            fix_rule_result.err().unwrap().to_string(),
            "You can not set add_bom with add mode."
        );
    }

    #[test]
    fn test_set_mode_as_remove_with_add_bom() {
        let args = Args {
            path: None,
            add: false,
            remove: true,
            add_strict: false,
            add_bom: Some(vec!["cs".to_string()]),
            remove_bom: None,
        };
        let expected_add = ["cs"]
            .iter()
            .map(|x| x.to_string())
            .collect::<HashSet<String>>();

        let fix_rule = parse_args(&args).unwrap();
        assert_eq!(fix_rule.mode, FixMode::Remove);
        assert_eq!(fix_rule.ext_add, expected_add);
    }

    #[test]
    fn test_set_mode_as_add_strict_with_add_bom() {
        let args = Args {
            path: None,
            add: false,
            remove: false,
            add_strict: true,
            add_bom: Some(vec!["cs".to_string()]),
            remove_bom: None,
        };

        let fix_rule_result = parse_args(&args);
        assert!(fix_rule_result.is_err());
        assert_eq!(
            fix_rule_result.err().unwrap().to_string(),
            "You can not set add_bom with add-strict mode."
        );
    }

    #[test]
    fn test_set_mode_as_add_with_remove_bom() {
        let args = Args {
            path: None,
            add: true,
            remove: false,
            add_strict: false,
            add_bom: None,
            remove_bom: Some(vec!["cs".to_string()]),
        };
        let expected_remove = vec![
            "html", "css", "svg", // web development
            "js", "ts", // programming language
            "md", // document
            "json", "toml", "yaml", "csv", "xml", // data
            "ini", "conf", "cfg", // config
            "sh", "bat", "ps1", // other
            "cs",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect::<HashSet<String>>();

        let fix_rule = parse_args(&args).unwrap();
        assert_eq!(fix_rule.mode, FixMode::Add);
        assert_eq!(fix_rule.ext_remove, expected_remove);
    }

    #[test]
    fn test_set_mode_as_remove_with_remove_bom() {
        let args = Args {
            path: None,
            add: false,
            remove: true,
            add_strict: false,
            add_bom: None,
            remove_bom: Some(vec!["cs".to_string()]),
        };

        let result_fix_rule = parse_args(&args);
        assert!(result_fix_rule.is_err());
        assert_eq!(
            result_fix_rule.err().unwrap().to_string(),
            "You can not set remove_bom with remove mode."
        );
    }

    #[test]
    fn test_set_mode_as_add_strict_with_remove_bom() {
        let args = Args {
            path: None,
            add: false,
            remove: false,
            add_strict: true,
            add_bom: None,
            remove_bom: Some(vec!["html".to_string()]),
        };
        let expected_remove = ["html"]
            .iter()
            .map(|x| x.to_string())
            .collect::<HashSet<String>>();

        let fix_rule = parse_args(&args).unwrap();
        assert_eq!(fix_rule.mode, FixMode::Add);
        assert_eq!(fix_rule.ext_remove, expected_remove);
    }

    #[test]
    fn test_set_mode_as_add_with_add_bom_and_remove_bom() {
        let args = Args {
            path: None,
            add: true,
            remove: false,
            add_strict: false,
            add_bom: Some(vec!["html".to_string()]),
            remove_bom: Some(vec!["html".to_string()]),
        };

        let fix_rule_result = parse_args(&args);
        assert!(fix_rule_result.is_err());
        assert_eq!(
            fix_rule_result.err().unwrap().to_string(),
            "You can not set add_bom with add mode."
        );
    }

    #[test]
    fn test_set_mode_as_remove_with_add_bom_and_remove_bom() {
        let args = Args {
            path: None,
            add: false,
            remove: true,
            add_strict: false,
            add_bom: Some(vec!["html".to_string()]),
            remove_bom: Some(vec!["html".to_string()]),
        };

        let fix_rule_result = parse_args(&args);
        assert!(fix_rule_result.is_err());
        assert_eq!(
            fix_rule_result.err().unwrap().to_string(),
            "You can not set remove_bom with remove mode."
        );
    }

    #[test]
    fn test_set_mode_as_add_strict_with_add_bom_and_remove_bom() {
        let args = Args {
            path: None,
            add: false,
            remove: false,
            add_strict: true,
            add_bom: Some(vec!["html".to_string()]),
            remove_bom: Some(vec!["html".to_string()]),
        };

        let fix_rule_result = parse_args(&args);
        assert!(fix_rule_result.is_err());
        assert_eq!(
            fix_rule_result.err().unwrap().to_string(),
            "You can not set add_bom with add-strict mode."
        );
    }
}
