use super::{Issue, IssueLevel};
use colored::Colorize;
use std::borrow::Cow;

#[derive(Debug)]
pub struct RootPackagePrivateFieldIssue;

impl RootPackagePrivateFieldIssue {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Issue for RootPackagePrivateFieldIssue {
    fn name(&self) -> &str {
        "root-package-private-field"
    }

    fn level(&self) -> IssueLevel {
        IssueLevel::Error
    }

    fn message(&self) -> String {
        format!(
            r#"  │ {{
  {}   "{}": "{}"   {}
  │ }}"#,
            "+".green(),
            "private".white(),
            "true".white(),
            "← missing private field.".green(),
        )
        .bright_black()
        .to_string()
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("The root package.json should be private to prevent accidentaly publishing it to a registry.")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = RootPackagePrivateFieldIssue::new();

        assert_eq!(issue.name(), "root-package-private-field");
        assert_eq!(issue.level(), IssueLevel::Error);
    }

    #[test]
    fn private_field_not_set() {
        let issue = RootPackagePrivateFieldIssue::new();

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());

        assert_eq!(issue.why(), "The root package.json should be private to prevent accidentaly publishing it to a registry.");
    }

    #[test]
    fn private_field_set_not_true() {
        let issue = RootPackagePrivateFieldIssue::new();

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
        assert_eq!(issue.why(), "The root package.json should be private to prevent accidentaly publishing it to a registry.");
    }
}
