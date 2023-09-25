use colored::Colorize;
use indexmap::IndexMap;
use std::{borrow::Cow, fmt::Display};

pub mod empty_dependencies;
pub mod mutiple_dependency_versions;
pub mod packages_without_package_json;
pub mod root_package_dependencies;
pub mod root_package_manager_field;
pub mod root_package_private_field;
pub mod types_in_dependencies;

pub const ERROR: &str = "⨯";
pub const WARNING: &str = "⚠️";
pub const SUCCESS: &str = "✓";

#[derive(Debug, PartialEq)]
pub enum IssueLevel {
    Error,
    Warning,
}

impl IssueLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueLevel::Error => "⨯ error",
            IssueLevel::Warning => "⚠️ warning",
        }
    }
}

impl Display for IssueLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.as_str();

        match self {
            IssueLevel::Error => write!(f, "{}", value.red()),
            IssueLevel::Warning => write!(f, "{}", value.yellow()),
        }
    }
}

pub trait Issue {
    fn name(&self) -> &str;
    fn level(&self) -> IssueLevel;
    fn message(&self) -> String;
    fn why(&self) -> Cow<'static, str>;
}

pub type BoxIssue = Box<dyn Issue>;

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum PackageType {
    None,
    Root,
    Package(String),
}

impl Display for PackageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageType::None => write!(f, "./"),
            PackageType::Root => write!(f, "./package.json"),
            PackageType::Package(name) => write!(f, "{}/package.json", name),
        }
    }
}

pub struct IssuesList<'a> {
    ignored_issues: &'a Vec<String>,
    issues: IndexMap<PackageType, Vec<BoxIssue>>,
}

impl<'a> IssuesList<'a> {
    pub fn new(ignored_issues: &'a Vec<String>) -> Self {
        Self {
            ignored_issues,
            issues: IndexMap::new(),
        }
    }

    pub fn add_raw(&mut self, package_type: PackageType, issue: BoxIssue) {
        if self.ignored_issues.contains(&issue.name().to_string()) {
            return;
        }

        self.issues.entry(package_type).or_default().push(issue);
    }

    pub fn add(&mut self, package_type: PackageType, issue: Option<BoxIssue>) {
        if let Some(issue) = issue {
            self.add_raw(package_type, issue);
        }
    }

    pub fn total_len(&self) -> usize {
        self.issues.values().flatten().collect::<Vec<_>>().len()
    }

    pub fn len_by_level(&self, level: IssueLevel) -> usize {
        self.issues
            .values()
            .flatten()
            .filter(|issue| issue.level() == level)
            .count()
    }
}

impl IntoIterator for IssuesList<'_> {
    type Item = (PackageType, Vec<BoxIssue>);
    type IntoIter = indexmap::map::IntoIter<PackageType, Vec<BoxIssue>>;

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
