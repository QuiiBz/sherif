use crate::args::Args;
use crate::printer::print_success;
use crate::rules::IssueLevel;
use clap::Parser;
use collect::{collect_issues, collect_packages};
use colored::Colorize;
use printer::{print_footer, print_issues};
use std::time::Instant;

mod args;
mod collect;
mod packages;
mod plural;
mod printer;
mod rules;

fn main() {
    let now = Instant::now();
    let args = Args::parse();
    let packages_list = match collect_packages(&args) {
        Ok(result) => result,
        Err(error) => {
            eprintln!();
            eprintln!(
                " {} {}",
                IssueLevel::Error,
                "Failed to collect packages".bold()
            );
            eprintln!("   {}", error.to_string().bright_black());
            std::process::exit(1);
        }
    };

    let total_packages = packages_list.packages.len();
    let issues = collect_issues(&args, packages_list);
    let total_issues = issues.total_len();

    if total_issues == 0 {
        print_success();
        return;
    }

    let warnings = issues.len_by_level(IssueLevel::Warning);
    let errors = issues.len_by_level(IssueLevel::Error);

    if let Err(error) = print_issues(issues) {
        eprintln!();
        eprintln!(" {} {}", IssueLevel::Error, "Failed to print issues".bold());
        eprintln!("   {}", error.to_string().bright_black());
        std::process::exit(1);
    }

    print_footer(total_issues, total_packages, warnings, errors, now);

    if errors > 0 {
        std::process::exit(1);
    }
}
