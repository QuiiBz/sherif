use super::{Issue, IssueLevel};
use crate::release_age::ConfigSource;
use colored::Colorize;
use std::borrow::Cow;

pub const RULE_NAME: &str = "unnecessary-release-age-exclude";

/// An entry of a bypass list that doesn't bypass anything anymore, with the age
/// of the most recent version it could unblock, in days.
#[derive(Debug)]
pub struct UnnecessaryExclude {
    pub raw: String,
    pub age_in_days: u64,
    /// Whether the entry doesn't pin a version, and is only unnecessary because
    /// the package hasn't published anything in a long time.
    pub dormant: bool,
}

#[derive(Debug)]
pub struct UnnecessaryReleaseAgeExcludeIssue {
    source: ConfigSource,
    /// The minimum release age, in seconds.
    max_age: u64,
    excludes: Vec<UnnecessaryExclude>,
}

impl UnnecessaryReleaseAgeExcludeIssue {
    pub fn new(source: ConfigSource, max_age: u64, excludes: Vec<UnnecessaryExclude>) -> Box<Self> {
        Box::new(Self {
            source,
            max_age,
            excludes,
        })
    }

    /// The minimum release age, in the unit used by the config file.
    fn configured_age(&self) -> u64 {
        match self.source {
            // Bun expresses the age in seconds, pnpm and Yarn in minutes.
            ConfigSource::Bunfig => self.max_age,
            _ => self.max_age / 60,
        }
    }
}

impl Issue for UnnecessaryReleaseAgeExcludeIssue {
    fn name(&self) -> &str {
        RULE_NAME
    }

    fn level(&self) -> IssueLevel {
        IssueLevel::Warning
    }

    fn message(&self) -> String {
        let excludes = self
            .excludes
            .iter()
            .map(|exclude| {
                format!(
                    "  {}  - '{}'   {}",
                    "-".red(),
                    exclude.raw.white(),
                    match exclude.dormant {
                        true => format!(
                            "← nothing published for {} days, currently bypasses nothing",
                            exclude.age_in_days
                        ),
                        false => format!(
                            "← published {} days ago, past the minimum release age",
                            exclude.age_in_days
                        ),
                    }
                    .red(),
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            r#"  │ {}: {}
  │ {}:   {}
{}"#,
            self.source.age_field(),
            self.configured_age(),
            self.source.exclude_field(),
            format!(
                "← {} bypasses the minimum release age...",
                self.source.file_name()
            )
            .blue(),
            excludes,
        )
        .bright_black()
        .to_string()
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("Packages excluded from the minimum release age bypass the cooldown that protects against compromised releases. These exclusions don't unblock anything anymore - they either pin versions that are past the minimum release age, or target packages that haven't published anything in a long time - and should be removed.")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = UnnecessaryReleaseAgeExcludeIssue::new(
            ConfigSource::PnpmWorkspace,
            1440 * 60,
            vec![
                UnnecessaryExclude {
                    raw: "nx@21.6.5".to_string(),
                    age_in_days: 42,
                    dormant: false,
                },
                UnnecessaryExclude {
                    raw: "left-pad".to_string(),
                    age_in_days: 3045,
                    dormant: true,
                },
            ],
        );

        assert_eq!(issue.name(), RULE_NAME);
        assert_eq!(issue.level(), IssueLevel::Warning);

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }

    #[test]
    fn bun() {
        let issue = UnnecessaryReleaseAgeExcludeIssue::new(
            ConfigSource::Bunfig,
            86400,
            vec![UnnecessaryExclude {
                raw: "typescript".to_string(),
                age_in_days: 12,
                dormant: false,
            }],
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }
}
