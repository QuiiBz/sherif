use crate::{
    plural::Pluralize,
    rules::{IssueLevel, IssuesList, ERROR, SUCCESS, WARNING},
};
use anyhow::Result;
use colored::Colorize;
use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};
use std::io::Write;
use std::time::Instant;

pub fn print_success() {
    println!();
    println!("{}", format!("{} No issues found", SUCCESS).green());
}

pub fn print_error(title: &str, message: &str) {
    eprintln!();
    eprintln!(" {} {}", IssueLevel::Error, title.bold());
    eprintln!("   {}", message.bright_black());
}

pub fn print_issues(issues: IssuesList) -> Result<()> {
    // Lock stdout manually instead of in every `println`
    // calls, since we might have a lot of them.
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    for (package_type, issues) in issues {
        writeln!(lock)?;
        writeln!(
            lock,
            "{} found in {}:",
            "issue".plural(issues.len()),
            package_type.to_string().bold(),
        )?;

        for issue in issues {
            writeln!(lock)?;
            writeln!(
                lock,
                " {} {} {}",
                issue.level().to_string().bold(),
                issue.why().bold(),
                issue.name().bright_black(),
            )?;
            writeln!(lock, "{}", issue.message())?;
        }
    }

    Ok(())
}

pub fn print_footer(
    total_issues: usize,
    total_packages: usize,
    warnings: usize,
    errors: usize,
    fixed: usize,
    start: Instant,
) {
    println!();
    println!(
        "{} found {} across {} in {:?}.",
        "issue".plural(total_issues),
        format!(
            "({} {}, {} {}, {} {})",
            errors, ERROR, warnings, WARNING, fixed, SUCCESS
        )
        .bright_black(),
        "package".plural(total_packages),
        start.elapsed(),
    );
    println!(
        "{}",
        " Note: use `-i` to ignore dependencies, `-r` to ignore rules, `-p` to ignore packages, and `-f` to autofix fixable issues."
            .bright_black()
    );
}

pub fn get_render_config() -> RenderConfig {
    let mut render_config = RenderConfig::default_colored()
        .with_prompt_prefix(Styled::new("✓").with_fg(Color::DarkGrey))
        .with_help_message(StyleSheet::new().with_fg(Color::DarkGrey))
        .with_highlighted_option_prefix(Styled::new(" → ").with_fg(Color::LightCyan))
        .with_canceled_prompt_indicator(Styled::new("✗").with_fg(Color::LightRed));
    render_config.answered_prompt_prefix = Styled::new("✓").with_fg(Color::LightGreen);
    render_config
}
