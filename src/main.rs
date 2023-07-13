#![allow(unused)]

mod checker;
mod explorer;
mod fix_rule;
mod fixer;

use clap::Parser;
use std::path::{Path, PathBuf};

fn main() {
    let args = Args::parse();
    let fix_rule = fix_rule::parse_fix_rule(&args).unwrap();
    let files =
        explorer::get_file_list(&PathBuf::from(args.path.unwrap_or_else(|| ".".to_string())));
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    path: Option<String>,
    #[arg(long, required = false)]
    add: bool,
    #[arg(long, required = false)]
    remove: bool,
    #[arg(long, required = false)]
    add_strict: bool,
    #[arg(long, required = false)]
    add_bom: Option<Vec<String>>,
    #[arg(long, required = false)]
    remove_bom: Option<Vec<String>>,
}
