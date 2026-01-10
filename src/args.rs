#[cfg(test)]
use crate::packages::Config;
use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Clone, ValueEnum, Deserialize)]
pub enum AutofixSelect {
    Highest,
    Lowest,
}

impl Display for AutofixSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutofixSelect::Highest => write!(f, "highest"),
            AutofixSelect::Lowest => write!(f, "lowest"),
        }
    }
}

#[derive(Debug, Parser, Deserialize, Clone)]
pub struct Args {
    /// Path to the monorepo root.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Fix the issues automatically, if possible.
    #[arg(long, short)]
    pub fix: bool,

    /// When using `--fix` with the `multiple-dependency-versions` rule, automatically select the highest or lower version of the dependency.
    #[arg(long, short)]
    pub select: Option<AutofixSelect>,

    /// Don't run your package manager's install command when autofixing.
    #[arg(long)]
    pub no_install: bool,

    /// Fail with a non-zero exit code if any warnings are found.
    #[arg(long)]
    pub fail_on_warnings: bool,

    /// Ignore the `multiple-dependency-versions` rule for the given dependency name and/or version.
    #[arg(long, short)]
    pub ignore_dependency: Vec<String>,

    /// Ignore rules for the given package name or path.
    #[arg(long, short = 'p')]
    pub ignore_package: Vec<String>,

    /// Ignore the given rule.
    #[arg(long, short = 'r')]
    pub ignore_rule: Vec<String>,
}

#[cfg(test)]
impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Config {
            fix: args.fix,
            select: args.select,
            no_install: args.no_install,
            fail_on_warnings: args.fail_on_warnings,
            ignore_dependency: args.ignore_dependency,
            ignore_package: args.ignore_package,
            ignore_rule: args.ignore_rule,
        }
    }
}
