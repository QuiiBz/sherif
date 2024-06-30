use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    /// Path to the monorepo root.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Fix the issues automatically, if possible.
    #[arg(long, short)]
    pub fix: bool,

    /// Don't run the package manager install command.
    #[arg(long)]
    pub no_install: bool,

    /// Ignore the `multiple-dependency-versions` rule for the given dependency name.
    #[arg(long, short)]
    pub ignore_dependency: Vec<String>,

    /// Ignore rules for the given package name or path.
    #[arg(long, short = 'p')]
    pub ignore_package: Vec<String>,

    /// Ignore the given rule.
    #[arg(long, short = 'r')]
    pub ignore_rule: Vec<String>,
}
