use super::{Issue, IssueLevel, PackageType};
use crate::json::{self};
use anyhow::Result;
use colored::Colorize;
use std::{borrow::Cow, fmt::Display, fs, path::PathBuf};

#[derive(Debug)]
pub enum DependencyKind {
    Dependencies,
    DevDependencies,
    PeerDependencies,
    OptionalDependencies,
}

impl Display for DependencyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyKind::Dependencies => write!(f, "dependencies"),
            DependencyKind::DevDependencies => write!(f, "devDependencies"),
            DependencyKind::PeerDependencies => write!(f, "peerDependencies"),
            DependencyKind::OptionalDependencies => write!(f, "optionalDependencies"),
        }
    }
}

#[derive(Debug)]
pub struct EmptyDependenciesIssue {
    dependency_kind: DependencyKind,
    fixed: bool,
}

impl EmptyDependenciesIssue {
    pub fn new(dependency_kind: DependencyKind) -> Box<Self> {
        Box::new(Self {
            dependency_kind,
            fixed: false,
        })
    }
}

impl Issue for EmptyDependenciesIssue {
    fn name(&self) -> &str {
        "empty-dependencies"
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
  {}   "{}": {}   {}
  │ }}"#,
            "-".red(),
            self.dependency_kind.to_string().white(),
            "{}".white(),
            "← field is empty.".red(),
        )
        .bright_black()
        .to_string()
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("package.json should not have empty dependencies fields.")
    }

    fn fix(&mut self, package_type: &PackageType) -> Result<()> {
        if let PackageType::Package(path) = package_type {
            let path = PathBuf::from(path).join("package.json");
            let value = fs::read_to_string(&path)?;
            let (mut value, indent, lineending) = json::deserialize::<serde_json::Value>(&value)?;
            let dependency = self.dependency_kind.to_string();

            if let Some(dependency_field) = value.get(&dependency) {
                if dependency_field.is_object() && dependency_field.as_object().unwrap().is_empty()
                {
                    value.as_object_mut().unwrap().remove(&dependency);

                    let value = json::serialize(&value, indent, lineending)?;
                    fs::write(path, value)?;

                    self.fixed = true;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = EmptyDependenciesIssue::new(DependencyKind::Dependencies);

        assert_eq!(issue.name(), "empty-dependencies");
        assert_eq!(issue.level(), IssueLevel::Error);
        assert_eq!(
            issue.why(),
            "package.json should not have empty dependencies fields."
        );
    }

    #[test]
    fn test_dependency_kind() {
        colored::control::set_override(false);

        let issue = EmptyDependenciesIssue::new(DependencyKind::Dependencies);
        insta::assert_snapshot!(issue.message());

        let issue = EmptyDependenciesIssue::new(DependencyKind::DevDependencies);
        insta::assert_snapshot!(issue.message());

        let issue = EmptyDependenciesIssue::new(DependencyKind::PeerDependencies);
        insta::assert_snapshot!(issue.message());

        let issue = EmptyDependenciesIssue::new(DependencyKind::OptionalDependencies);
        insta::assert_snapshot!(issue.message());
    }
}
