use super::{Issue, IssueLevel};
use std::borrow::Cow;

#[derive(Debug)]
pub struct DuplicatedDependencyFromRootIssue {
    name: String,
    package: String,
}

impl DuplicatedDependencyFromRootIssue {
    pub fn new(name: String, package: String) -> Box<Self> {
        Box::new(Self { name, package })
    }
}

impl Issue for DuplicatedDependencyFromRootIssue {
    fn name(&self) -> &str {
        "duplicated-dependency-from-root"
    }

    fn level(&self) -> IssueLevel {
        IssueLevel::Warning
    }

    fn message(&self) -> String {
        format!(
            "The `{}` dependency is duplicated from the root in {}/package.json.",
            self.name, self.package,
        )
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Borrowed("Dependencies declared in root `package.json` should not be duplicated in package's `package.json`")
    }
}
