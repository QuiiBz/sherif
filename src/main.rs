use crate::packages::Config;
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

    if args.fix && is_ci() {
        print_error(
            "Failed to fix issues",
            "Cannot fix issues inside a CI environment",
        );
        std::process::exit(1);
    }

    let packages_list = match collect_packages(&args) {
        Ok(result) => result,
        Err(error) => {
            print_error("Failed to collect packages", error.to_string().as_str());
            std::process::exit(1);
        }
    };

    let mut config = packages_list.root_package.get_config().unwrap_or_default();

    if args.fix {
        config.fix = Some(true);
    }

    if args.no_install {
        config.no_install = Some(true);
    }

    if let Some(select) = args.select {
        config.select = Some(select);
    }

    if args.fail_on_warnings {
        config.fail_on_warnings = Some(true);
    }

    if args.ignore_dependency.len() > 0 {
        config.ignore_dependency = Some(args.ignore_dependency);
    }

    if args.ignore_package.len() > 0 {
        config.ignore_package = Some(args.ignore_package);
    }

    if args.ignore_rule.len() > 0 {
        config.ignore_rule = Some(args.ignore_rule);
    }

    let total_packages = packages_list.packages.len();
    let mut issues = collect_issues(&config, packages_list);

    if args.fix {
        if let Some(autofix_select) = &config.select {
            println!(
                " {}",
                format!("Note: automatically selecting {} dependencies for `multiple-dependency-versions` rule...", autofix_select).bright_black(),
            );
            println!();
        }

        if let Err(error) = issues.fix() {
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
    if args.fix && !args.no_install && fixed > 0 {
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

    if errors > 0 || (args.fail_on_warnings && warnings > 0) {
        std::process::exit(1);
    }
}
