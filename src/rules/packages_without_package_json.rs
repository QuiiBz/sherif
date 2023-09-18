use super::{Issue, IssueLevel};
use crate::plural::Pluralize;
use std::borrow::Cow;

#[derive(Debug)]
pub struct PackagesWithoutPackageJsonIssue {
    packages: Vec<String>,
}

impl PackagesWithoutPackageJsonIssue {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
        }
    }

    pub fn add_package(&mut self, package: String) {
        self.packages.push(package);
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
        format!(
            "{} doesn't have a package.json file: {}",
            "package".plural(self.packages.len()),
            self.packages.join(", ")
        )
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("All packages defined in `workspaces` or `pnpm-workspace.yaml` should have a package.json file.")
    }

    fn to_packages_without_package_json_issue(
        &mut self,
    ) -> Option<&mut PackagesWithoutPackageJsonIssue> {
        Some(self)
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