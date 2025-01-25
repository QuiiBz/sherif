use super::Issue;
use crate::packages::semversion::SemVersion;
use colored::Colorize;
use indexmap::IndexMap;
use std::{borrow::Cow, fmt::Display, hash::Hash};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum SimilarDependency {
    React,
    NextJS,
    Turborepo,
    TanstackQuery,
}

impl Display for SimilarDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::React => write!(f, "React"),
            Self::NextJS => write!(f, "Next.js"),
            Self::Turborepo => write!(f, "Turborepo"),
            Self::TanstackQuery => write!(f, "Tanstack Query"),
        }
    }
}

impl TryFrom<&str> for SimilarDependency {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "react" | "react-dom" => Ok(Self::React),
            "next"
            | "@next/eslint-plugin-next"
            | "eslint-config-next"
            | "@next/bundle-analyzer"
            | "@next/third-parties"
            | "@next/mdx" => Ok(Self::NextJS),
            "turbo"
            | "turbo-ignore"
            | "eslint-config-turbo"
            | "eslint-plugin-turbo"
            | "@turbo/gen"
            | "@turbo/workspaces" => Ok(Self::Turborepo),
            "@tanstack/eslint-plugin-query"
            | "@tanstack/query-async-storage-persister"
            | "@tanstack/query-broadcast-client-experimental"
            | "@tanstack/query-core"
            | "@tanstack/query-devtools"
            | "@tanstack/query-persist-client-core"
            | "@tanstack/query-sync-storage-persister"
            | "@tanstack/react-query"
            | "@tanstack/react-query-devtools"
            | "@tanstack/react-query-persist-client"
            | "@tanstack/react-query-next-experimental"
            | "@tanstack/solid-query"
            | "@tanstack/solid-query-devtools"
            | "@tanstack/solid-query-persist-client"
            | "@tanstack/svelte-query"
            | "@tanstack/svelte-query-devtools"
            | "@tanstack/svelte-query-persist-client"
            | "@tanstack/vue-query"
            | "@tanstack/vue-query-devtools"
            | "@tanstack/angular-query-devtools-experimental"
            | "@tanstack/angular-query-experimental" => Ok(Self::TanstackQuery),
            _ => Err(anyhow::anyhow!("Unknown similar dependency")),
        }
    }
}

#[derive(Debug)]
pub struct UnsyncSimilarDependenciesIssue {
    r#type: SimilarDependency,
    versions: IndexMap<SemVersion, String>,
    fixed: bool,
}

impl UnsyncSimilarDependenciesIssue {
    pub fn new(r#type: SimilarDependency, versions: IndexMap<SemVersion, String>) -> Box<Self> {
        Box::new(Self {
            r#type,
            versions,
            fixed: false,
        })
    }
}

impl Issue for UnsyncSimilarDependenciesIssue {
    fn name(&self) -> &str {
        "unsync-similar-dependencies"
    }

    fn level(&self) -> super::IssueLevel {
        match self.fixed {
            true => super::IssueLevel::Fixed,
            false => super::IssueLevel::Error,
        }
    }

    fn message(&self) -> String {
        let deps = self
            .versions
            .iter()
            .map(|(version, dependency)| {
                format!(
                    r#"  {}      "{}": "{}""#,
                    "~".yellow(),
                    dependency.white(),
                    version.to_string().yellow()
                )
            })
            .collect::<Vec<String>>()
            .join(",\n");

        format!(
            r#"  │ {{
  │   "{}": {{
{}
  │   }}
  │ }}"#,
            "dependencies".white(),
            deps,
        )
        .bright_black()
        .to_string()
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{} dependencies aren't synced.", self.r#type))
    }

    fn fix(&mut self, _package_type: &super::PackageType) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::IssueLevel;

    #[test]
    fn test() {
        let versions = vec![
            (SemVersion::parse("1.0.0").unwrap(), "react".to_string()),
            (SemVersion::parse("2.0.0").unwrap(), "react-dom".to_string()),
        ]
        .into_iter()
        .collect();

        let issue = UnsyncSimilarDependenciesIssue::new(SimilarDependency::React, versions);

        assert_eq!(issue.name(), "unsync-similar-dependencies");
        assert_eq!(issue.level(), IssueLevel::Error);
        assert_eq!(issue.versions.len(), 2);
        assert_eq!(issue.why(), "React dependencies aren't synced.".to_string());
    }

    #[test]
    fn basic() {
        let versions = vec![
            (SemVersion::parse("1.0.0").unwrap(), "react".to_string()),
            (SemVersion::parse("2.0.0").unwrap(), "react-dom".to_string()),
        ]
        .into_iter()
        .collect();

        let issue = UnsyncSimilarDependenciesIssue::new(SimilarDependency::React, versions);

        colored::control::set_override(false);
        insta::assert_snapshot!(issue.message());
    }
}
