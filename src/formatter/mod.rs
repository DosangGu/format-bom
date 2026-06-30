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
