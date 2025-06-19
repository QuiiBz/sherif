use super::{Issue, IssueLevel, PackageType};
use crate::{
    args::AutofixSelect, json, packages::semversion::SemVersion, printer::get_render_config,
};
use anyhow::Result;
use colored::Colorize;
use indexmap::IndexMap;
use inquire::Select;
use std::{borrow::Cow, fs, path::PathBuf};

#[derive(Debug)]
pub struct MultipleDependencyVersionsIssue {
    name: String,
    versions: IndexMap<String, SemVersion>,
    select: Option<AutofixSelect>,
    fixed: bool,
}

impl MultipleDependencyVersionsIssue {
    pub fn new(
        name: String,
        mut versions: IndexMap<String, SemVersion>,
        select: Option<AutofixSelect>,
    ) -> Box<Self> {
        versions.sort_by(|_, a, _, b| b.cmp(a));

        Box::new(Self {
            name,
            versions,
            select,
            fixed: false,
        })
    }

    fn get_autofix_version(&self) -> Result<Option<String>> {
        let mut sorted_versions = self.versions.values().collect::<Vec<_>>();
        sorted_versions.sort_by(|a, b| b.cmp(a));

        if let Some(select) = &self.select {
            let autofix_version = match select {
                AutofixSelect::Highest => sorted_versions.first().map(|v| v.to_string()),
                AutofixSelect::Lowest => sorted_versions.last().map(|v| v.to_string()),
            };
            Ok(autofix_version)
        } else {
            let message = format!("Select the version of {} to use:", self.name.bold());
            let mut versions = sorted_versions
                .iter()
                .map(|version| format_version(version, &self.versions, true))
                .collect::<Vec<_>>();
            versions.dedup();

            let autofix_version = Select::new(&message, versions)
                .with_render_config(get_render_config())
                .with_help_message("Enter to select, Esc to skip")
                .prompt_skippable()?;
            let autofix_version = autofix_version.map(|select| {
                select
                    .split_once(' ')
                    .expect("Please report this as a bug")
                    .0
                    .to_string()
            });
            Ok(autofix_version)
        }
    }
}

fn format_version(
    version: &SemVersion,
    versions: &IndexMap<String, SemVersion>,
    skip_version_color: bool,
) -> String {
    let (version, indicator) = if version == versions.first().unwrap().1 {
        (version.to_string().green(), "↑ highest".green())
    } else if version == versions.last().unwrap().1 {
        (version.to_string().red(), "↓ lowest".red())
    } else {
        (version.to_string().yellow(), "∼ between".yellow())
    };
    let version = match skip_version_color {
        true => version.clear(),
        false => version,
    };

    format!("{}   {}", version, indicator)
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
                let mut end = common_path.pop().unwrap();

                if end == "." {
                    end = "./";
                }

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
        if let Some(autofix_version) = self.get_autofix_version()? {
            for package in self.versions.keys() {
                let path = PathBuf::from(package).join("package.json");
                let value = fs::read_to_string(&path)?;
                let (mut value, indent, lineending) =
                    json::deserialize::<serde_json::Value>(&value)?;

                if let Some(dependencies) = value.get_mut("dependencies") {
                    let dependencies = dependencies.as_object_mut().unwrap();

                    if let Some(dependency) = dependencies.get_mut(&self.name) {
                        *dependency = serde_json::Value::String(autofix_version.clone());
                    }
                }

                if let Some(dev_dependencies) = value.get_mut("devDependencies") {
                    let dev_dependencies = dev_dependencies.as_object_mut().unwrap();

                    if let Some(dev_dependency) = dev_dependencies.get_mut(&self.name) {
                        *dev_dependency = serde_json::Value::String(autofix_version.clone());
                    }
                }

                let value = json::serialize(&value, indent, lineending)?;
                fs::write(path, value)?;
            }

            self.fixed = true;
        }

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
                "./packages/package-a".into() => SemVersion::parse("1.2.3").unwrap(),
                "./packages/package-b".into() => SemVersion::parse("1.2.4").unwrap(),
                "./package-c".into() => SemVersion::parse("1.2.5").unwrap(),
            },
            None,
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
    fn root() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            indexmap::indexmap! {
                "./".into() => SemVersion::parse("5.6.3").unwrap(),
                "./packages/package-a".into() => SemVersion::parse("1.2.3").unwrap(),
                "./packages/package-b".into() => SemVersion::parse("3.1.6").unwrap(),
            },
            None,
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }

    #[test]
    fn order_single() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            indexmap::indexmap! {
                "./package-a".into() => SemVersion::parse("1.2.3").unwrap(),
            },
            None,
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }

    #[test]
    fn order_multiple() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            indexmap::indexmap! {
                "./apps/package-a".into() => SemVersion::parse("5.6.3").unwrap(),
                "./apps/package-b".into() => SemVersion::parse("1.2.3").unwrap(),
                "./packages/package-c".into() => SemVersion::parse("3.1.6").unwrap(),
            },
            None,
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }

    #[test]
    fn order_prerelease() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            indexmap::indexmap! {
                "./apps/package-a".into() => SemVersion::parse("5.0.0-next.4").unwrap(),
                "./apps/package-b".into() => SemVersion::parse("5.0.0-next.3").unwrap(),
                "./packages/package-c".into() => SemVersion::parse("5.0.0-next.6").unwrap(),
            },
            None,
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }

    #[test]
    fn exact_and_range() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            indexmap::indexmap! {
                "./apps/package-a".into() => SemVersion::parse("5.6.3").unwrap(),
                "./apps/package-b".into() => SemVersion::parse("^1.2.3").unwrap(),
                "./packages/package-c".into() => SemVersion::parse("~3.1.6").unwrap(),
            },
            None,
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }

    #[test]
    fn dedupe() {
        let issue = MultipleDependencyVersionsIssue::new(
            "test".to_string(),
            indexmap::indexmap! {
                "./package-a".into() => SemVersion::parse("1.2.3").unwrap(),
                "./packages/package-b".into() => SemVersion::parse("3.1.6").unwrap(),
                "./packages/package-c".into() => SemVersion::parse("3.1.6").unwrap(),
            },
            None,
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }
}
