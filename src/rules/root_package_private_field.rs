use super::{Issue, IssueLevel};
use colored::Colorize;
use std::borrow::Cow;

#[derive(Debug)]
pub struct RootPackagePrivateFieldIssue {
    field_exists: bool,
}

impl RootPackagePrivateFieldIssue {
    pub fn new(field_exists: bool) -> Box<Self> {
        Box::new(Self { field_exists })
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
        match self.field_exists {
            true => format!(
                "./package.json `{}` field is set to {}, but should be {}.",
                "private".blue(),
                "false".red(),
                "true".green()
            ),
            false => format!("./package.json is missing `{}` field.", "private".blue()),
        }
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
        let issue = RootPackagePrivateFieldIssue::new(true);

        assert_eq!(issue.name(), "root-package-private-field");
        assert_eq!(issue.level(), IssueLevel::Error);
    }

    #[test]
    fn private_field_not_set() {
        let issue = RootPackagePrivateFieldIssue::new(false);

        colored::control::set_override(false);
        assert_eq!(
            issue.message(),
            "./package.json is missing `private` field."
        );
    }

    #[test]
    fn private_field_set_not_true() {
        let issue = RootPackagePrivateFieldIssue::new(true);

        colored::control::set_override(false);
        assert_eq!(
            issue.message(),
            "./package.json `private` field is set to false, but should be true."
        );
    }
}
