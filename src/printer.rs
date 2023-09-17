use crate::{
    plural::Pluralize,
    rules::{IssuesList, ERROR, IGNORED, SUCCESS, WARNING},
};
use colored::Colorize;

pub fn print_success() {
    println!();
    println!("{}", format!("{} No issues found", SUCCESS).green());
}

pub fn print_header(
    total_issues: usize,
    total_packages: usize,
    warnings: usize,
    errors: usize,
    ignored: usize,
) {
    println!();
    println!(
        "{} found {} across {}:",
        "issue".plural(total_issues),
        format!(
            "({} {}, {} {}, {} {})",
            errors, ERROR, warnings, WARNING, ignored, IGNORED,
        )
        .bright_black(),
        "package".plural(total_packages)
    );
}

pub fn print_issues(issues: IssuesList) {
    for issue in issues {
        let pad = " ".repeat(issue.level().as_str().len());

        println!();
        println!(
            " {} {}",
            issue.level().to_string().bold(),
            issue.message().bold()
        );
        println!(
            "{}{}",
            pad,
            format!("{}: {}", issue.name(), issue.why()).bright_black(),
        );
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
