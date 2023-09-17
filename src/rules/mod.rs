use colored::Colorize;
use std::{borrow::Cow, fmt::Display};

pub mod empty_dependencies;
pub mod mutiple_dependency_versions;
pub mod root_package_dependencies;
pub mod root_package_manager_field;
pub mod root_package_private_field;

pub const ERROR: &str = "⨯";
pub const WARNING: &str = "⚠️";
pub const IGNORED: &str = "⊙";
pub const SUCCESS: &str = "✓";

#[derive(Debug, PartialEq)]
pub enum IssueLevel {
    Error,
    Warning,
    Ignored,
}

impl IssueLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueLevel::Error => "⨯ error",
            IssueLevel::Warning => "⚠️ warning",
            IssueLevel::Ignored => "⊙ ignored",
        }
    }
}

impl Display for IssueLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.as_str();

        match self {
            IssueLevel::Error => write!(f, "{}", value.red()),
            IssueLevel::Warning => write!(f, "{}", value.yellow()),
            IssueLevel::Ignored => write!(f, "{}", value.bright_black()),
        }
    }
}

pub trait Issue {
    fn name(&self) -> &str;
    fn level(&self) -> IssueLevel;
    fn message(&self) -> String;
    fn why(&self) -> Cow<'static, str>;
}

pub struct IssuesList<'a> {
    ignored_issues: &'a Vec<String>,
    issues: Vec<Box<dyn Issue>>,
}

impl<'a> IssuesList<'a> {
    pub fn new(ignored_issues: &'a Vec<String>) -> Self {
        Self {
            ignored_issues,
            issues: Vec::new(),
        }
    }

    pub fn add_raw(&mut self, issue: Box<dyn Issue>) {
        if self.ignored_issues.contains(&issue.name().to_string()) {
            return;
        }

        self.issues.push(issue);
    }

    pub fn add(&mut self, issue: Option<Box<dyn Issue>>) {
        if let Some(issue) = issue {
            self.add_raw(issue);
        }
    }

    pub fn total_len(&self) -> usize {
        self.issues.len()
    }

    pub fn len_by_level(&self, level: IssueLevel) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.level() == level)
            .count()
    }
}

impl IntoIterator for IssuesList<'_> {
    type Item = Box<dyn Issue>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.issues.into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::{
        root_package_dependencies::RootPackageDependenciesIssue,
        root_package_manager_field::RootPackageManagerFieldIssue,
    };

    #[test]
    fn add_issues() {
        let ignored_issues = Vec::new();
        let mut issues = IssuesList::new(&ignored_issues);

        issues.add(Some(RootPackageManagerFieldIssue::new()));
        assert_eq!(issues.total_len(), 1);

        issues.add_raw(RootPackageManagerFieldIssue::new());
        assert_eq!(issues.total_len(), 2);

        issues.add(None);
        assert_eq!(issues.total_len(), 2);
    }

    #[test]
    fn add_ignored() {
        let ignored_issues = vec!["root-package-manager-field".to_string()];
        let mut issues = IssuesList::new(&ignored_issues);

        issues.add_raw(RootPackageManagerFieldIssue::new());
        assert_eq!(issues.total_len(), 0);

        issues.add_raw(RootPackageDependenciesIssue::new());
        assert_eq!(issues.total_len(), 1);
    }

    #[test]
    fn len_by_level() {
        let ignored_issues = Vec::new();
        let mut issues = IssuesList::new(&ignored_issues);

        issues.add_raw(RootPackageManagerFieldIssue::new());
        issues.add_raw(RootPackageDependenciesIssue::new());
        issues.add_raw(RootPackageDependenciesIssue::new());
        issues.add_raw(RootPackageDependenciesIssue::new());

        assert_eq!(issues.len_by_level(IssueLevel::Error), 1);
        assert_eq!(issues.len_by_level(IssueLevel::Warning), 3);
        assert_eq!(issues.len_by_level(IssueLevel::Ignored), 0);
    }
}
