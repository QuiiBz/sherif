use super::{Issue, IssueLevel};
use std::borrow::Cow;

#[derive(Debug)]
pub struct PackagesWithoutPackageJsonIssue {
    package: String,
}

impl PackagesWithoutPackageJsonIssue {
    pub fn new(package: String) -> Box<Self> {
        Box::new(Self { package })
    }
}

impl Issue for PackagesWithoutPackageJsonIssue {
    fn name(&self) -> &str {
        "packages-without-package-json"
    }

    fn level(&self) -> IssueLevel {
        IssueLevel::Warning
    }

    fn message(&self) -> String {
        format!("  {}/package.json doesn't exists.", self.package)
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("All packages in the workspace should have a package.json file.")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = PackagesWithoutPackageJsonIssue::new();

        assert_eq!(issue.name(), "packages-without-package-json");
        assert_eq!(issue.level(), IssueLevel::Warning);
    }

    #[test]
    fn single_package() {
        let mut issue = PackagesWithoutPackageJsonIssue::new();
        issue.add_package("test".to_string());

        colored::control::set_override(false);
        assert_eq!(
            issue.message(),
            "1 package doesn't have a package.json file: test"
        );
    }

    #[test]
    fn multiple_packages() {
        let mut issue = PackagesWithoutPackageJsonIssue::new();
        issue.add_package("test".to_string());
        issue.add_package("test-2".to_string());
        issue.add_package("test-3".to_string());

        colored::control::set_override(false);
        assert_eq!(
            issue.message(),
            "3 packages doesn't have a package.json file: test, test-2, test-3"
        );
    }
}
