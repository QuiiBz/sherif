use crate::{
    plural::Pluralize,
    rules::{IssuesList, ERROR, SUCCESS, WARNING},
};
use colored::Colorize;

pub fn print_success() {
    println!();
    println!("{}", format!("{} No issues found", SUCCESS).green());
}

pub fn print_header(total_issues: usize, total_packages: usize, warnings: usize, errors: usize) {
    println!();
    println!(
        "{} found {} across {}:",
        "issue".plural(total_issues),
        format!("({} {}, {} {})", errors, ERROR, warnings, WARNING,).bright_black(),
        "package".plural(total_packages)
    );
}

pub fn print_issues(issues: IssuesList) {
    for (package_type, issues) in issues {
        println!();
        println!(
            "{} found in {}:",
            "issue".plural(issues.len()),
            package_type.to_string().bold(),
        );

        for issue in issues {
            println!();
            println!(
                " {} {}",
                issue.level().to_string().bold(),
                issue.why().bold(),
            );
            println!("{}", issue.message());
        }
    }
}

pub fn print_footer() {
    println!();
    println!(
        "{}",
        "Note: use `-i` to ignore dependencies, `-r` to ignore rules, and `-p` to ignore packages"
            .black()
    );
}
