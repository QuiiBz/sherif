use super::{Issue, IssueLevel};
use colored::Colorize;
use indexmap::IndexMap;
use semver::{Comparator, Op, Prerelease, VersionReq};
use std::{borrow::Cow, cmp::Ordering};

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
    versions: IndexMap<String, VersionReq>,
}

impl MultipleDependencyVersionsIssue {
    pub fn new(name: String, mut versions: IndexMap<String, VersionReq>) -> Box<Self> {
        versions.sort_by(|_, a, _, b| {
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

        Box::new(Self { name, versions })
    }
}

impl Issue for MultipleDependencyVersionsIssue {
    fn name(&self) -> &str {
        "multiple-dependency-versions"
    }

    fn level(&self) -> IssueLevel {
        IssueLevel::Error
    }

    fn message(&self) -> String {
        let mut group = vec![];

        self.versions
            .iter()
            .map(|(package, version)| {
                let mut common_path = package.split('/').collect::<Vec<_>>();
                let end = common_path.pop().unwrap();

                let (version, indicator) = if version == self.versions.last().unwrap().1 {
                    (version.to_string().green(), "↑ highest".green())
                } else if version == self.versions.first().unwrap().1 {
                    (version.to_string().red(), "↓ lowest".red())
                } else {
                    (version.to_string().yellow(), "∼ between".yellow())
                };

                let version_pad = " ".repeat(if end.len() > 16 { 3 } else { 16 - end.len() });

                if group.is_empty() || group != common_path {
                    let root = common_path.join("/").bright_black();
                    group = common_path;

                    return format!(
                        "  {}
      {}{}{}   {}",
                        root,
                        end.bright_black(),
                        version_pad,
                        version,
                        indicator
                    );
                }

                group = common_path;

                format!(
                    "      {}{}{}   {}",
                    end.bright_black(),
                    version_pad,
                    version,
                    indicator
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "Dependency {} has multiple versions defined in the workspace.",
            self.name
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            indexmap::indexmap! {
                "packages/package-a".into() => VersionReq::parse("1.2.3").unwrap(),
                "packages/package-b".into() => VersionReq::parse("1.2.4").unwrap(),
                "package-c".into() => VersionReq::parse("1.2.5").unwrap(),
            },
        );

        assert_eq!(issue.name(), "multiple-dependency-versions");
        assert_eq!(issue.level(), IssueLevel::Error);
        assert_eq!(issue.versions.len(), 3);
        assert_eq!(
            issue.why(),
            "Dependency test has multiple versions defined in the workspace.".to_string()
        );
    }

    #[test]
    fn order_single() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            indexmap::indexmap! {
                "package-a".into() => VersionReq::parse("1.2.3").unwrap(),
            },
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }

    #[test]
    fn order_multiple() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            indexmap::indexmap! {
                "apps/package-a".into() => VersionReq::parse("5.6.3").unwrap(),
                "apps/package-b".into() => VersionReq::parse("1.2.3").unwrap(),
                "packages/package-c".into() => VersionReq::parse("3.1.6").unwrap(),
            },
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }

    #[test]
    fn dedupe() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            indexmap::indexmap! {
                "package-a".into() => VersionReq::parse("1.2.3").unwrap(),
                "packages/package-b".into() => VersionReq::parse("3.1.6").unwrap(),
                "packages/package-c".into() => VersionReq::parse("3.1.6").unwrap(),
            },
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }
}
