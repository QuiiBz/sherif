use crate::printer::print_success;
use crate::rules::IssueLevel;
use crate::{args::Args, printer::print_error};
use clap::Parser;
use collect::{collect_issues, collect_packages};
use colored::Colorize;
use printer::{print_footer, print_issues};
use std::time::Instant;

mod args;
mod collect;
mod install;
mod json;
mod packages;
mod plural;
mod printer;
mod rules;

fn is_ci() -> bool {
    std::env::var("CI").is_ok()
}

fn main() {
    let now = Instant::now();
    let args = Args::parse();

    let packages_list = match collect_packages(&args.path) {
        Ok(result) => result,
        Err(error) => {
            print_error("Failed to collect packages", error.to_string().as_str());
            std::process::exit(1);
        }
    };

    let mut config = packages_list.root_package.get_config().unwrap_or_default();
    config.merge(args);

    if config.fix && is_ci() {
        print_error(
            "Failed to fix issues",
            "Cannot fix issues inside a CI environment",
        );
        std::process::exit(1);
    }

    let total_packages = packages_list.packages.len();
    let mut issues = collect_issues(&config, packages_list);

    if config.fix {
        if let Some(autofix_select) = &config.select {
            println!(
                " {}",
                format!("Note: automatically selecting {} dependencies for `multiple-dependency-versions` rule...", autofix_select).bright_black(),
            );
            println!();
        }

        if let Err(error) = issues.fix(&args.path) {
            print_error("Failed to fix issues", error.to_string().as_str());
            std::process::exit(1);
        }
    }

    let total_issues = issues.total_len();

    if total_issues == 0 {
        print_success();
        return;
    }

    let warnings = issues.len_by_level(IssueLevel::Warning);
    let errors = issues.len_by_level(IssueLevel::Error);
    let fixed = issues.len_by_level(IssueLevel::Fixed);

    // Only run the install command if we allow it and we fixed some issues.
    if config.fix && !config.no_install && fixed > 0 {
        if let Err(error) = install::install() {
            print_error("Failed to install packages", error.to_string().as_str());
            std::process::exit(1);
        }
    }

    if let Err(error) = print_issues(issues) {
        print_error("Failed to print issues", error.to_string().as_str());
        std::process::exit(1);
    }

    print_footer(total_issues, total_packages, warnings, errors, fixed, now);

    if errors > 0 || (config.fail_on_warnings && warnings > 0) {
        std::process::exit(1);
    }
}
