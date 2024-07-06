use super::{Issue, IssueLevel, PackageType};
use crate::json;
use anyhow::Result;
use colored::Colorize;
use indexmap::IndexMap;
use std::{borrow::Cow, fs, path::PathBuf};

#[derive(Debug)]
pub struct TypesInDependenciesIssue {
    packages: Vec<String>,
    fixed: bool,
}

impl TypesInDependenciesIssue {
    pub fn new(packages: Vec<String>) -> Box<Self> {
        Box::new(Self {
            packages,
            fixed: false,
        })
    }
}

impl Issue for TypesInDependenciesIssue {
    fn name(&self) -> &str {
        "types-in-dependencies"
    }

    fn level(&self) -> IssueLevel {
        match self.fixed {
            true => IssueLevel::Fixed,
            false => IssueLevel::Error,
        }
    }

    fn message(&self) -> String {
        let before = self
            .packages
            .iter()
            .map(|package| format!(r#"  {}      "{}": "...","#, "-".red(), package.white()))
            .collect::<Vec<String>>()
            .join("\n");

        let after = self
            .packages
            .iter()
            .map(|package| format!(r#"  {}      "{}": "...","#, "+".green(), package.white()))
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            r#"  │ {{
  │   "{}": "{}",     {}
  │   ...
  {}   "{}": {{      {}
{}
  {}   }},
  │   ...
  {}   "{}": {{   {}
{}
  {}   }}
  │ }}"#,
            "private".white(),
            "true".white(),
            "← package is private...".blue(),
            "-".red(),
            "dependencies".white(),
            "← but has @types/* in dependencies...".red(),
            before,
            "-".red(),
            "+".green(),
            "devDependencies".white(),
            "← instead of devDependencies.".green(),
            after,
            "+".green(),
        )
        .bright_black()
        .to_string()
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("Private packages shouldn't have @types/* in dependencies.")
    }

    fn fix(&mut self, package_type: &PackageType) -> Result<()> {
        if let PackageType::Package(path) = package_type {
            let path = PathBuf::from(path).join("package.json");
            let value = fs::read_to_string(&path)?;
            let (mut value, indent, lineending) = json::deserialize::<serde_json::Value>(&value)?;

            let dependencies = value
                .get_mut("dependencies")
                .unwrap()
                .as_object_mut()
                .unwrap();
            let mut dependencies_to_add = IndexMap::new();

            for package in &self.packages {
                if let Some(version) = dependencies.remove(package) {
                    dependencies_to_add.insert(package.clone(), version);
                }
            }

            // The package.json file might not have a devDependencies field.
            let dev_dependencies = match value.get_mut("devDependencies") {
                Some(dev_dependencies) => dev_dependencies,
                None => {
                    value.as_object_mut().unwrap().insert(
                        "devDependencies".into(),
                        serde_json::Value::Object(serde_json::Map::new()),
                    );

                    value.get_mut("devDependencies").unwrap()
                }
            };

            let dev_dependencies = dev_dependencies.as_object_mut().unwrap();

            for (package, version) in dependencies_to_add {
                dev_dependencies.insert(package, version);
            }

            let value = json::serialize(&value, indent, lineending)?;
            fs::write(path, value)?;

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
        let issue =
            TypesInDependenciesIssue::new(vec!["@types/react".into(), "@types/react-dom".into()]);

        assert_eq!(issue.name(), "types-in-dependencies");
        assert_eq!(issue.level(), IssueLevel::Error);

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
        assert_eq!(
            issue.why(),
            "Private packages shouldn't have @types/* in dependencies."
        );
    }
}
