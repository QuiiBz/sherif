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
        format!(
            "{}/package.json is private but has `{}` dependencies in `{}` instead of `{}`.",
            self.package,
            "@types/*".blue(),
            "dependencies".red(),
            "devDependencies".green(),
        )
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "Private packages shouldn't have `@types/*` in `dependencies`: {}",
            self.packages.join(", ")
        ))
    }
}
