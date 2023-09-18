use super::{Issue, IssueLevel};
use colored::Colorize;
use std::{borrow::Cow, fmt::Display};

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
            DependencyKind::Dependencies => write!(f, "{}", "dependencies".blue()),
            DependencyKind::DevDependencies => write!(f, "{}", "devDependencies".blue()),
            DependencyKind::PeerDependencies => write!(f, "{}", "peerDependencies".blue()),
            DependencyKind::OptionalDependencies => write!(f, "{}", "optionalDependencies".blue()),
        }
    }
}

#[derive(Debug)]
pub struct EmptyDependenciesIssue {
    package: String,
    dependency_kind: DependencyKind,
}

impl EmptyDependenciesIssue {
    pub fn new(package: String, dependency_kind: DependencyKind) -> Box<Self> {
        Box::new(Self {
            package,
            dependency_kind,
        })
    }
}

impl Issue for EmptyDependenciesIssue {
    fn name(&self) -> &str {
        "empty-dependencies"
    }

    fn level(&self) -> IssueLevel {
        IssueLevel::Error
    }

    fn message(&self) -> String {
        format!(
            "{}/package.json `{}` field is empty.",
            self.package,
            self.dependency_kind.to_string().blue()
        )
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("package.json should not have empty dependencies fields.")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = EmptyDependenciesIssue::new("test".to_string(), DependencyKind::Dependencies);

        assert_eq!(issue.name(), "empty-dependencies");
        assert_eq!(issue.level(), IssueLevel::Error);
    }

    #[test]
    fn test_dependency_kind() {
        colored::control::set_override(false);

        let issue = EmptyDependenciesIssue::new("test".to_string(), DependencyKind::Dependencies);
        assert_eq!(
            issue.message(),
            "test/package.json `dependencies` field is empty."
        );

        let issue =
            EmptyDependenciesIssue::new("test".to_string(), DependencyKind::DevDependencies);
        assert_eq!(
            issue.message(),
            "test/package.json `devDependencies` field is empty."
        );

        let issue =
            EmptyDependenciesIssue::new("test".to_string(), DependencyKind::PeerDependencies);
        assert_eq!(
            issue.message(),
            "test/package.json `peerDependencies` field is empty."
        );

        let issue =
            EmptyDependenciesIssue::new("test".to_string(), DependencyKind::OptionalDependencies);
        assert_eq!(
            issue.message(),
            "test/package.json `optionalDependencies` field is empty."
        );
    }
}
