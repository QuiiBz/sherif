use super::{Issue, IssueLevel, PackageType};
use anyhow::Result;
use std::{borrow::Cow, fs, path::PathBuf};

#[derive(Debug)]
pub struct PackagesWithoutPackageJsonIssue {
    package: String,
    fixed: bool,
}

impl PackagesWithoutPackageJsonIssue {
    pub fn new(package: String) -> Box<Self> {
        Box::new(Self {
            package,
            fixed: false,
        })
    }
}

impl Issue for PackagesWithoutPackageJsonIssue {
    fn name(&self) -> &str {
        "packages-without-package-json"
    }

    fn level(&self) -> IssueLevel {
        match self.fixed {
            true => IssueLevel::Fixed,
            false => IssueLevel::Warning,
        }
    }

    fn message(&self) -> String {
        format!("   {}/package.json doesn't exist.", self.package)
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("All packages matching the workspace should have a package.json file.")
    }

    fn fix(&mut self, root: &PathBuf, _package_type: &PackageType) -> Result<()> {
        let path = root.join(&self.package).join("package.json");
        let package_name = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let value = serde_json::json!({
            "name": package_name,
            "version": "0.0.0",
            "private": true,
        });

        let value = serde_json::to_string_pretty(&value)?;
        fs::write(path, value)?;

        self.fixed = true;

        Ok(())
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
        assert_eq!(issue.message(), "   test/package.json doesn't exist.");
        assert_eq!(
            issue.why(),
            "All packages matching the workspace should have a package.json file."
        );
    }
}
