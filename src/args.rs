use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    /// Path to the monorepo root.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Ignore the `multiple-dependency-versions` rule for the given dependency name.
    #[arg(long, short)]
    pub ignore_dependency: Vec<String>,

    /// Ignore rules for the given package name.
    #[arg(long, short = 'p')]
    pub ignore_package: Vec<String>,

    /// Ignore the given rule.
    #[arg(long, short = 'r')]
    pub ignore_rule: Vec<String>,
}
