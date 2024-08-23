use super::{empty_dependencies::DependencyKind, Issue, IssueLevel, PackageType};
use anyhow::Result;
use colored::Colorize;
use std::borrow::Cow;

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
        // TODO
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
