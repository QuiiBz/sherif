use colored::Colorize;

use super::{Issue, IssueLevel};
use std::borrow::Cow;

#[derive(Debug)]
pub struct TypesInDependenciesIssue {
    package: String,
    packages: Vec<String>,
}

impl TypesInDependenciesIssue {
    pub fn new(package: String, packages: Vec<String>) -> Box<Self> {
        Box::new(Self { package, packages })
    }
}

impl Issue for TypesInDependenciesIssue {
    fn name(&self) -> &str {
        "types-in-dependencies"
    }

    fn level(&self) -> IssueLevel {
        IssueLevel::Error
    }

    fn message(&self) -> String {
        let before = self
            .packages
            .iter()
            .map(|package| format!(r#"{}      "{}": "...""#, "-".red(), package.white()))
            .collect::<Vec<String>>()
            .join("\n");

        let after = self
            .packages
            .iter()
            .map(|package| format!(r#"{}      "{}": "...""#, "+".green(), package.white()))
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let issue = TypesInDependenciesIssue::new(
            "test".into(),
            vec!["@types/react".into(), "@types/react-dom".into()],
        );

        assert_eq!(issue.name(), "types-in-dependencies");
        assert_eq!(issue.level(), IssueLevel::Error);

        colored::control::set_override(false);
        assert_eq!(
            issue.message(),
            "test/package.json is private but has `@types/*` dependencies in `dependencies` instead of `devDependencies`."
        );
        assert_eq!(
            issue.why(),
            "Private packages shouldn't have `@types/*` in `dependencies`: @types/react, @types/react-dom"
        );
    }
}
