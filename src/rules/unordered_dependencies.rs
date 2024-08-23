use super::{empty_dependencies::DependencyKind, Issue, IssueLevel, PackageType};
use crate::json;
use anyhow::Result;
use colored::Colorize;
use std::{borrow::Cow, fs, path::PathBuf};

#[derive(Debug)]
pub struct UnorderedDependenciesIssue {
    dependency_kind: DependencyKind,
    fixed: bool,
}

impl UnorderedDependenciesIssue {
    pub fn new(dependency_kind: DependencyKind) -> Box<Self> {
        Box::new(Self {
            dependency_kind,
            fixed: false,
        })
    }

    pub fn sort(&mut self, path: PathBuf) -> Result<()> {
        let value = fs::read_to_string(&path)?;
        let (mut value, indent, lineending) = json::deserialize::<serde_json::Value>(&value)?;
        let dependency = self.dependency_kind.to_string();

        if let Some(dependency_field) = value.get(&dependency) {
            if dependency_field.is_object() {
                let mut keys = dependency_field
                    .as_object()
                    .unwrap()
                    .keys()
                    .collect::<Vec<_>>();
                keys.sort();

                let mut sorted = serde_json::Map::new();
                for key in keys {
                    sorted.insert(key.to_string(), dependency_field[key].clone());
                }

                value
                    .as_object_mut()
                    .unwrap()
                    .insert(dependency, serde_json::Value::Object(sorted));

                let value = json::serialize(&value, indent, lineending)?;
                fs::write(path, value)?;

                self.fixed = true;
            }
        }

        Ok(())
    }
}

impl Issue for UnorderedDependenciesIssue {
    fn name(&self) -> &str {
        "unordered-dependencies"
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
  {}   "{}": {{   {}
  {}     ...
  {}   }}
  │ }}"#,
            "-".red(),
            self.dependency_kind.to_string().white(),
            "← keys aren't sorted.".red(),
            "-".red(),
            "-".red(),
        )
        .bright_black()
        .to_string()
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "{} should be ordered alphabetically.",
            self.dependency_kind
        ))
    }

    fn fix(&mut self, package_type: &PackageType) -> Result<()> {
        if let PackageType::Package(path) = package_type {
            let path = PathBuf::from(path).join("package.json");
            self.sort(path)?;
        } else if let PackageType::Root = package_type {
            let path = PathBuf::from("package.json");
            self.sort(path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = UnorderedDependenciesIssue::new(DependencyKind::Dependencies);

        assert_eq!(issue.name(), "unordered-dependencies");
        assert_eq!(issue.level(), IssueLevel::Error);
        assert_eq!(
            issue.why(),
            "dependencies should be ordered alphabetically."
        );
    }

    #[test]
    fn test_dependency_kind() {
        colored::control::set_override(false);

        let issue = UnorderedDependenciesIssue::new(DependencyKind::Dependencies);
        insta::assert_snapshot!(issue.message());

        let issue = UnorderedDependenciesIssue::new(DependencyKind::DevDependencies);
        insta::assert_snapshot!(issue.message());

        let issue = UnorderedDependenciesIssue::new(DependencyKind::PeerDependencies);
        insta::assert_snapshot!(issue.message());

        let issue = UnorderedDependenciesIssue::new(DependencyKind::OptionalDependencies);
        insta::assert_snapshot!(issue.message());
    }
}
