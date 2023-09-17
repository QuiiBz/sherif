use crate::args::Args;
use crate::printer::{print_header, print_success};
use crate::rules::IssueLevel;
use clap::Parser;
use collect::{collect_issues, collect_packages};
use colored::Colorize;
use printer::{print_footer, print_issues};

mod args;
mod collect;
mod packages;
mod plural;
mod printer;
mod rules;

fn main() {
    let args = Args::parse();
    let (root_package, packages) = match collect_packages(&args) {
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

    let issues = collect_issues(&args, &root_package, &packages);

    let total_issues = issues.total_len();
    let total_packages = packages.len();

    if total_issues == 0 {
        print_success();
        return;
    }

    let warnings = issues.len_by_level(IssueLevel::Warning);
    let errors = issues.len_by_level(IssueLevel::Error);
    let ignored = issues.len_by_level(IssueLevel::Ignored);

    print_header(total_issues, total_packages, warnings, errors, ignored);
    print_issues(issues);

    if errors > 0 {
        print_footer();
        std::process::exit(1);
    }
}
