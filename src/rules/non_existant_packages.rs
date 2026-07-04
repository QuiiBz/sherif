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
                true => {
                    format!(
                        "  {}  - '{}'   {}",
                        "-".red(),
                        package.white(),
                        "← but this one doesn't match any package".red(),
                    )
                }
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

    fn fix(&mut self, package_type: &PackageType) -> Result<()> {
        if let PackageType::None = package_type {
            match self.pnpm_workspace {
                true => {
                    let path = PathBuf::from("pnpm-workspace.yaml");
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
                    let path = PathBuf::from("package.json");
                    let value = fs::read_to_string(&path)?;
                    let (mut value, indent, lineending) =
                        json::deserialize::<serde_json::Value>(&value)?;

                    let workspaces = value.get_mut("workspaces").unwrap();

                    // Leave the issue unfixed (so it is still reported) when the
                    // `workspaces` field has an unexpected shape we can't edit.
                    if retain_existing_workspace_paths(workspaces, &self.paths) {
                        let value = json::serialize(&value, indent, lineending)?;
                        fs::write(path, value)?;

                        self.fixed = true;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Removes the given `paths` from a root `package.json` `workspaces` field.
///
/// The field can be either an array (`["packages/*"]`) or an object holding a
/// `packages` array (Yarn/Bun: `{ "packages": ["packages/*"] }`).
///
/// Returns `true` when the field had a known shape and was edited, and `false`
/// when the shape is unrecognized and was left untouched (so the caller can
/// keep reporting the issue instead of silently claiming a fix).
fn retain_existing_workspace_paths(workspaces: &mut serde_json::Value, paths: &[String]) -> bool {
    let workspace_paths = match workspaces {
        serde_json::Value::Array(workspace_paths) => Some(workspace_paths),
        serde_json::Value::Object(object) => object
            .get_mut("packages")
            .and_then(serde_json::Value::as_array_mut),
        _ => None,
    };

    match workspace_paths {
        Some(workspace_paths) => {
            workspace_paths.retain(|package| match package.as_str() {
                Some(package) => !paths.contains(&package.to_string()),
                None => true,
            });

            true
        }
        None => false,
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

    #[test]
    fn retain_paths_array_workspaces() {
        let mut workspaces = serde_json::json!(["apps/*", "empty/*", "docs"]);

        let edited =
            retain_existing_workspace_paths(&mut workspaces, &["empty/*".into(), "docs".into()]);

        assert!(edited);
        assert_eq!(workspaces, serde_json::json!(["apps/*"]));
    }

    #[test]
    fn retain_paths_object_workspaces() {
        // Yarn/Bun object form: `{ "packages": [...] }`. Previously panicked.
        let mut workspaces = serde_json::json!({
            "packages": ["apps/*", "empty/*", "docs"],
            "catalog": { "react": "^19.0.0" },
        });

        let edited =
            retain_existing_workspace_paths(&mut workspaces, &["empty/*".into(), "docs".into()]);

        assert!(edited);
        assert_eq!(
            workspaces,
            serde_json::json!({
                "packages": ["apps/*"],
                "catalog": { "react": "^19.0.0" },
            })
        );
    }

    #[test]
    fn retain_paths_unknown_shape_is_noop() {
        let mut workspaces = serde_json::json!("packages/*");

        let edited = retain_existing_workspace_paths(&mut workspaces, &["packages/*".into()]);

        assert!(!edited);
        assert_eq!(workspaces, serde_json::json!("packages/*"));
    }
}
