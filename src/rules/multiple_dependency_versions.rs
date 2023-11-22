use super::{Issue, IssueLevel, PackageType};
use anyhow::Result;
use colored::Colorize;
use indexmap::IndexMap;
use inquire::{
    ui::{Color, RenderConfig, Styled},
    Select,
};
use semver::{Comparator, Op, Prerelease, VersionReq};
use std::{borrow::Cow, cmp::Ordering, collections::HashSet, fs, path::PathBuf};

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
    fixed: bool,
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

        Box::new(Self {
            name,
            versions,
            fixed: false,
        })
    }
}

fn format_version(
    version: &VersionReq,
    versions: &IndexMap<String, VersionReq>,
    skip_version_color: bool,
) -> String {
    let (version, indicator) = if version == versions.last().unwrap().1 {
        (version.to_string().green(), "↑ highest".green())
    } else if version == versions.first().unwrap().1 {
        (version.to_string().red(), "↓ lowest".red())
    } else {
        (version.to_string().yellow(), "∼ between".yellow())
    };

    return format!(
        "{}   {}",
        if skip_version_color {
            version.clear()
        } else {
            version
        },
        indicator
    );
}

impl Issue for MultipleDependencyVersionsIssue {
    fn name(&self) -> &str {
        "multiple-dependency-versions"
    }

    fn level(&self) -> IssueLevel {
        match self.fixed {
            true => IssueLevel::Fixed,
            false => IssueLevel::Error,
        }
    }

    fn message(&self) -> String {
        let mut group = vec![];

        self.versions
            .iter()
            .map(|(package, version)| {
                let mut common_path = package.split('/').collect::<Vec<_>>();
                let end = common_path.pop().unwrap();

                let formatted_version = format_version(version, &self.versions, false);
                let version_pad = " ".repeat(if end.len() >= 26 { 3 } else { 26 - end.len() });

                if group.is_empty() || group != common_path {
                    let root = common_path.join("/").bright_black();
                    group = common_path;

                    if group.len() == 1 && group[0] == "." {
                        let root = format!("{}{}", "./".bright_black(), end.bright_black());

                        return format!("  {}  {}{}", root, version_pad, formatted_version);
                    }

                    return format!(
                        "  {}
      {}{}{}",
                        root,
                        end.bright_black(),
                        version_pad,
                        formatted_version
                    );
                }

                group = common_path;

                format!(
                    "      {}{}{}",
                    end.bright_black(),
                    version_pad,
                    formatted_version
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

    fn fix(&mut self, _package_type: &PackageType) -> Result<()> {
        let message = format!("Select the version of {} to use:", self.name).bold();

        let versions = self
            .versions
            .values()
            .map(|version| format_version(version, &self.versions, true))
            .collect::<HashSet<_>>();
        let versions = versions.iter().collect::<Vec<_>>();
        let mut render_config = RenderConfig::default_colored()
            .with_prompt_prefix(Styled::new("?").with_fg(Color::DarkGrey))
            .with_highlighted_option_prefix(Styled::new(" → ").with_fg(Color::LightCyan));
        render_config.answered_prompt_prefix = Styled::new("✓").with_fg(Color::LightGreen);

        println!();
        let select = Select::new(&message, versions)
            .with_render_config(render_config)
            .without_help_message()
            .prompt()?;

        let version = select
            .split_once(" ")
            .expect("Please report this as a bug")
            .0
            .to_string();

        for package in self.versions.keys() {
            let path = PathBuf::from(package).join("package.json");
            let value = fs::read_to_string(&path)?;
            let mut value = serde_json::from_str::<serde_json::Value>(&value)?;

            if let Some(dependencies) = value.get_mut("dependencies") {
                let dependencies = dependencies.as_object_mut().unwrap();

                if let Some(dependency) = dependencies.get_mut(&self.name) {
                    *dependency = serde_json::Value::String(version.to_string());
                }
            }

            if let Some(dev_dependencies) = value.get_mut("devDependencies") {
                let dev_dependencies = dev_dependencies.as_object_mut().unwrap();

                if let Some(dev_dependency) = dev_dependencies.get_mut(&self.name) {
                    *dev_dependency = serde_json::Value::String(version.to_string());
                }
            }

            let value = serde_json::to_string_pretty(&value)?;
            fs::write(path, value)?;
        }

        self.fixed = true;

        Ok(())
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
                "./packages/package-a".into() => VersionReq::parse("1.2.3").unwrap(),
                "./packages/package-b".into() => VersionReq::parse("1.2.4").unwrap(),
                "./package-c".into() => VersionReq::parse("1.2.5").unwrap(),
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
                "./package-a".into() => VersionReq::parse("1.2.3").unwrap(),
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
                "./apps/package-a".into() => VersionReq::parse("5.6.3").unwrap(),
                "./apps/package-b".into() => VersionReq::parse("1.2.3").unwrap(),
                "./packages/package-c".into() => VersionReq::parse("3.1.6").unwrap(),
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
                "./package-a".into() => VersionReq::parse("1.2.3").unwrap(),
                "./packages/package-b".into() => VersionReq::parse("3.1.6").unwrap(),
                "./packages/package-c".into() => VersionReq::parse("3.1.6").unwrap(),
            },
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }
}
