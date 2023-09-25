use super::{Issue, IssueLevel};
use colored::Colorize;
use indexmap::IndexMap;
use semver::VersionReq;
use std::borrow::Cow;

#[derive(Debug)]
pub struct MultipleDependencyVersionsIssue {
    name: String,
    versions: IndexMap<String, VersionReq>,
}

impl MultipleDependencyVersionsIssue {
    pub fn new(name: String, versions: IndexMap<String, VersionReq>) -> Box<Self> {
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
                    (version.to_string().green(), "⬆️ highest".green())
                } else if version == self.versions.first().unwrap().1 {
                    (version.to_string().red(), "⬇️ lowest".red())
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
