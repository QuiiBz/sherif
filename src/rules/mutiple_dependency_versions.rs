use super::{Issue, IssueLevel};
use crate::plural::Pluralize;
use colored::Colorize;
use semver::{Comparator, Op, Prerelease, VersionReq};
use std::{borrow::Cow, cmp::Ordering};

fn extract_version(version: Option<&VersionReq>) -> String {
    version.map_or("x.x.x".to_string(), |version| version.to_string())
}

const DEFAULT_COMPARATOR: Comparator = Comparator {
    op: Op::Exact,
    major: 0,
    minor: None,
    patch: None,
    pre: Prerelease::EMPTY,
};

#[derive(Debug)]
pub struct MultipleDependencyVersionsIssue {
    name: String,
    versions: Vec<VersionReq>,
    ignored: bool,
}

impl MultipleDependencyVersionsIssue {
    pub fn new(name: String, mut versions: Vec<VersionReq>, ignored: bool) -> Box<Self> {
        versions.sort_by(|a, b| {
            let a_comparator = a.comparators.get(0).cloned().unwrap_or(DEFAULT_COMPARATOR);
            let b_comparator = b.comparators.get(0).cloned().unwrap_or(DEFAULT_COMPARATOR);

            let mut ordering = Ordering::Equal;

            ordering = match a_comparator.patch.cmp(&b_comparator.patch) {
                Ordering::Equal => ordering,
                ordering => ordering,
            };

            ordering = match a_comparator.minor.cmp(&b_comparator.minor) {
                Ordering::Equal => ordering,
                ordering => ordering,
            };

            ordering = match a_comparator.major.cmp(&b_comparator.major) {
                Ordering::Equal => ordering,
                ordering => ordering,
            };

            ordering
        });

        versions.dedup();

        Box::new(Self {
            name,
            versions,
            ignored,
        })
    }
}

impl Issue for MultipleDependencyVersionsIssue {
    fn name(&self) -> &str {
        "multiple-dependency-versions"
    }

    fn level(&self) -> IssueLevel {
        match self.ignored {
            true => IssueLevel::Ignored,
            false => IssueLevel::Error,
        }
    }

    fn message(&self) -> String {
        let lowest_version = extract_version(self.versions.first());
        let highest_version = extract_version(self.versions.last());

        match self.ignored {
            true => format!(
                "The `{}` dependency has multiple versions, {} being the lowest and {} the highest.",
                self.name,
                lowest_version,
                highest_version,
            ).bright_black().to_string(),
            false => format!(
                "The `{}` dependency has multiple versions, {} being the lowest and {} the highest.",
                self.name.blue(),
                lowest_version.red(),
                highest_version.green(),
            )
        }
    }

    fn why(&self) -> Cow<'static, str> {
        let versions = self
            .versions
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>();
        let total_versions = self.versions.len();

        Cow::Owned(format!(
            "{} has {}: {}",
            self.name,
            "version".plural(total_versions),
            versions.join(", ")
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_version() {
        assert_eq!(extract_version(None), "x.x.x");
        assert_eq!(
            extract_version(Some(&VersionReq::parse("1.2.3").unwrap())),
            "^1.2.3"
        );
        assert_eq!(
            extract_version(Some(&VersionReq::parse("^1.2.3").unwrap())),
            "^1.2.3"
        );
    }

    #[test]
    fn test() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            vec![
                VersionReq::parse("1.2.3").unwrap(),
                VersionReq::parse("1.2.4").unwrap(),
                VersionReq::parse("1.2.5").unwrap(),
            ],
            false,
        );

        assert_eq!(issue.name(), "multiple-dependency-versions");
        assert_eq!(issue.level(), IssueLevel::Error);
        assert_eq!(issue.versions.len(), 3);
        assert!(!issue.ignored);
    }

    #[test]
    fn order_single() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            vec![VersionReq::parse("1.2.3").unwrap()],
            false,
        );

        colored::control::set_override(false);
        assert_eq!(issue.message(), "The `test` dependency has multiple versions, ^1.2.3 being the lowest and ^1.2.3 the highest.");
        assert_eq!(issue.why(), "test has 1 version: ^1.2.3".to_string());
    }

    #[test]
    fn order_multiple() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            vec![
                VersionReq::parse("5.6.3").unwrap(),
                VersionReq::parse("1.2.3").unwrap(),
                VersionReq::parse("3.1.6").unwrap(),
            ],
            false,
        );

        colored::control::set_override(false);
        assert_eq!(issue.message(), "The `test` dependency has multiple versions, ^1.2.3 being the lowest and ^5.6.3 the highest.");
        assert_eq!(
            issue.why(),
            "test has 3 versions: ^1.2.3, ^3.1.6, ^5.6.3".to_string()
        );
    }

    #[test]
    fn dedupe() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            vec![
                VersionReq::parse("1.2.3").unwrap(),
                VersionReq::parse("3.1.6").unwrap(),
                VersionReq::parse("3.1.6").unwrap(),
            ],
            false,
        );

        colored::control::set_override(false);
        assert_eq!(issue.message(), "The `test` dependency has multiple versions, ^1.2.3 being the lowest and ^3.1.6 the highest.");
        assert_eq!(
            issue.why(),
            "test has 2 versions: ^1.2.3, ^3.1.6".to_string()
        );
    }
}
