use super::{Issue, IssueLevel, PackageType};
use crate::json;
use anyhow::Result;
use colored::Colorize;
use std::{borrow::Cow, fs, path::PathBuf};

#[derive(Debug)]
pub struct RootPackagePrivateFieldIssue {
    fixed: bool,
}

impl RootPackagePrivateFieldIssue {
    pub fn new() -> Box<Self> {
        Box::new(Self { fixed: false })
    }
}

impl Issue for RootPackagePrivateFieldIssue {
    fn name(&self) -> &str {
        "root-package-private-field"
    }

    fn level(&self) -> IssueLevel {
        match self.fixed {
            true => IssueLevel::Fixed,
            false => IssueLevel::Error,
        }
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

    fn fix(&mut self, package_type: &PackageType) -> Result<()> {
        if let PackageType::Root = package_type {
            let path = PathBuf::from("package.json");
            let value = fs::read_to_string(&path)?;
            let (mut value, indent, lineending) = json::deserialize::<serde_json::Value>(&value)?;

            value
                .as_object_mut()
                .unwrap()
                .insert("private".to_string(), serde_json::Value::Bool(true));

            let value = json::serialize(&value, indent, lineending)?;
            fs::write(path, value)?;

            self.fixed = true;
        }

        Ok(())
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
