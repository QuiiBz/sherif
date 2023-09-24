use super::{Issue, IssueLevel};
use colored::Colorize;
use std::borrow::Cow;

#[derive(Debug)]
pub struct RootPackageManagerFieldIssue;

impl RootPackageManagerFieldIssue {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl Issue for RootPackageManagerFieldIssue {
    fn name(&self) -> &str {
        "root-package-manager-field"
    }

    fn level(&self) -> IssueLevel {
        IssueLevel::Error
    }

    fn message(&self) -> String {
        format!(
            r#"  │ {{
  {}   "{}": "..."   {}
  │ }}"#,
            "+".green(),
            "packageManager".white(),
            "← missing packageManager field.".green(),
        )
        .bright_black()
        .to_string()
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("The root package.json should specify the package manager and version to use. Useful for tools like corepack.")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = RootPackageManagerFieldIssue::new();

        assert_eq!(issue.name(), "root-package-manager-field");
        assert_eq!(issue.level(), IssueLevel::Error);

        colored::control::set_override(false);
        assert_eq!(
            issue.message(),
            "./package.json is missing `packageManager` field."
        );
    }
}
