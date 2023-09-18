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
            "./package.json shouldn't have any `{}` , only `{}`.",
            "dependencies".red(),
            "devDependencies".green()
        )
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("The root package.json is private, so making a distinction is useless.")
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
        assert_eq!(
            issue.message(),
            "./package.json shouldn't have any `dependencies` , only `devDependencies`."
        );
    }
}
