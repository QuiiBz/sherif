use super::{Issue, IssueLevel, PackageType};
use crate::json;
use anyhow::Result;
use colored::Colorize;
use std::{borrow::Cow, fs, path::PathBuf};

#[derive(Debug)]
pub struct NonExistantPackagesIssue {
    pnpm_workspace: bool,
    packages_list: Vec<String>,
    paths: Vec<String>,
    fixed: bool,
}

impl NonExistantPackagesIssue {
    pub fn new(pnpm_workspace: bool, packages_list: Vec<String>, paths: Vec<String>) -> Box<Self> {
        Box::new(Self {
            pnpm_workspace,
            packages_list,
            paths,
            fixed: false,
        })
    }

    fn pnpm_message(&self) -> String {
        let workspaces = self
            .packages_list
            .iter()
            .map(|package| match self.paths.contains(package) {
                true => format!(
                    "  {}  - '{}'   {}",
                    "-".red(),
                    package.white(),
                    "← but this one doesn't match any package".red(),
                ),
                false => format!("  │  - '{}'", package),
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            r#"  │ packages:   {}
{}"#,
            "← Workspace has paths defined...".blue(),
            workspaces,
        )
        .bright_black()
        .to_string()
    }

    fn package_message(&self) -> String {
        let workspaces = self
            .packages_list
            .iter()
            .map(|package| match self.paths.contains(package) {
                true => format!(
                    r#"  {}     "{}",   {}"#,
                    "-".red(),
                    package.white(),
                    "← but this one doesn't match any package".red(),
                ),
                false => format!(r#"  │     "{}","#, package),
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            r#"  │ {{
  │   "workspaces": [   {}
{}
  │   ],
  │ }}"#,
            "← Workspace has paths defined...".blue(),
            workspaces,
        )
        .bright_black()
        .to_string()
    }
}

impl Issue for NonExistantPackagesIssue {
    fn name(&self) -> &str {
        "non-existant-packages"
    }

    fn level(&self) -> IssueLevel {
        match self.fixed {
            true => IssueLevel::Fixed,
            false => IssueLevel::Warning,
        }
    }

    fn message(&self) -> String {
        match self.pnpm_workspace {
            true => self.pnpm_message(),
            false => self.package_message(),
        }
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("All paths defined in the workspace should match at least one package.")
    }

    fn fix(&mut self, root: &PathBuf, package_type: &PackageType) -> Result<()> {
        if let PackageType::None = package_type {
            match self.pnpm_workspace {
                true => {
                    let path = root.join("pnpm-workspace.yaml");
                    let value = fs::read_to_string(&path)?;
                    let mut value = serde_yaml::from_str::<serde_yaml::Value>(&value)?;

                    value
                        .get_mut("packages")
                        .unwrap()
                        .as_sequence_mut()
                        .unwrap()
                        .retain(|package| {
                            let package = package.as_str().unwrap().to_string();

                            !self.paths.contains(&package)
                        });

                    let value = serde_yaml::to_string(&value)?;
                    fs::write(path, value)?;

                    self.fixed = true;
                }
                false => {
                    let path = root.join("package.json");
                    let value = fs::read_to_string(&path)?;
                    let (mut value, indent, lineending) =
                        json::deserialize::<serde_json::Value>(&value)?;

                    value
                        .get_mut("workspaces")
                        .unwrap()
                        .as_array_mut()
                        .unwrap()
                        .retain(|package| {
                            let package = package.as_str().unwrap().to_string();

                            !self.paths.contains(&package)
                        });

                    let value = json::serialize(&value, indent, lineending)?;
                    fs::write(path, value)?;

                    self.fixed = true;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = NonExistantPackagesIssue::new(
            true,
            vec![
                "apps/*".into(),
                "packages/*".into(),
                "empty/*".into(),
                "docs".into(),
            ],
            vec!["empty/*".into(), "docs".into()],
        );

        assert_eq!(issue.name(), "non-existant-packages");
        assert_eq!(issue.level(), IssueLevel::Warning);
        assert_eq!(
            issue.why(),
            "All paths defined in the workspace should match at least one package."
        );
    }

    #[test]
    fn test_pnpm_workspace() {
        let issue = NonExistantPackagesIssue::new(
            true,
            vec![
                "apps/*".into(),
                "packages/*".into(),
                "empty/*".into(),
                "docs".into(),
            ],
            vec!["empty/*".into(), "docs".into()],
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }

    #[test]
    fn test_package_workspace() {
        let issue = NonExistantPackagesIssue::new(
            false,
            vec![
                "apps/*".into(),
                "packages/*".into(),
                "empty/*".into(),
                "docs".into(),
            ],
            vec!["empty/*".into(), "docs".into()],
        );

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }
}
