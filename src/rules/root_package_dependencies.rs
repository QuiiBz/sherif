use super::{Issue, IssueLevel};
use colored::Colorize;
use std::borrow::Cow;

#[derive(Debug)]
pub struct RootPackageDependenciesIssue;

impl RootPackageDependenciesIssue {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Issue for RootPackageDependenciesIssue {
    fn name(&self) -> &str {
        "root-package-dependencies"
    }

    fn level(&self) -> IssueLevel {
        IssueLevel::Warning
    }

    fn message(&self) -> String {
        format!(
            r#"  │ {{
  │   "{}": "{}",     {}
  │   ...
  {}   "{}": {{      {}
  {}      ...
  {}   }},
  │   ...
  {}   "{}": {{   {}
  {}      ...
  {}   }}
  │ }}"#,
            "private".white(),
            "true".white(),
            "← root package is private...".blue(),
            "-".red(),
            "dependencies".white(),
            "← but has dependencies...".red(),
            "-".red(),
            "-".red(),
            "+".green(),
            "devDependencies".white(),
            "← instead of devDependencies.".green(),
            "+".green(),
            "+".green(),
        )
        .bright_black()
        .to_string()
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("The root package.json is private and should only have devDependencies. Declare dependencies in each package.")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = RootPackageDependenciesIssue::new();

        assert_eq!(issue.name(), "root-package-dependencies");
        assert_eq!(issue.level(), IssueLevel::Warning);

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
        assert_eq!(
            issue.why(),
            "The root package.json is private and should only have devDependencies. Declare dependencies in each package."
        );
    }
}
