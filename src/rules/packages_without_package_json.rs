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
        format!("   {}/package.json doesn't exists.", self.package)
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("All packages matching the workspace should have a package.json file.")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = PackagesWithoutPackageJsonIssue::new("test".to_string());

        assert_eq!(issue.name(), "packages-without-package-json");
        assert_eq!(issue.level(), IssueLevel::Warning);

        colored::control::set_override(false);
        assert_eq!(issue.message(), "   test/package.json doesn't exists.");
        assert_eq!(
            issue.why(),
            "All packages matching the workspace should have a package.json file."
        );
    }
}
