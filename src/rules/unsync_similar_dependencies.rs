use super::Issue;
use crate::packages::semversion::SemVersion;
use colored::Colorize;
use indexmap::IndexMap;
use std::{borrow::Cow, fmt::Display};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum SimilarDependency {
    NextJS,
    Turborepo,
}

impl Display for SimilarDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NextJS => write!(f, "Next.js"),
            Self::Turborepo => write!(f, "Turborepo"),
        }
    }
}

impl TryFrom<&str> for SimilarDependency {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            // Next.js
            "next" => Ok(Self::NextJS),
            "@next/eslint-plugin-next" => Ok(Self::NextJS),
            "eslint-config-next" => Ok(Self::NextJS),
            "@next/bundle-analyzer" => Ok(Self::NextJS),
            "@next/third-parties" => Ok(Self::NextJS),
            "@next/mdx" => Ok(Self::NextJS),
            // Turborepo
            "turbo" => Ok(Self::Turborepo),
            "turbo-ignore" => Ok(Self::Turborepo),
            "eslint-config-turbo" => Ok(Self::Turborepo),
            "eslint-plugin-turbo" => Ok(Self::Turborepo),
            "@turbo/gen" => Ok(Self::Turborepo),
            "@turbo/workspaces" => Ok(Self::Turborepo),
            _ => Err(anyhow::anyhow!("Unknown similar dependency")),
        }
    }
}

#[derive(Debug)]
pub struct UnsyncSimilarDependenciesIssue {
    r#type: SimilarDependency,
    versions: IndexMap<String, Vec<(String, SemVersion)>>,
    fixed: bool,
}

impl UnsyncSimilarDependenciesIssue {
    pub fn new(
        r#type: SimilarDependency,
        versions: IndexMap<String, Vec<(String, SemVersion)>>,
    ) -> Box<Self> {
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
        self.versions
            .iter()
            .map(|(dependency, versions)| {
                // let version_pad = " ".repeat(if dependency.len() >= 26 {
                //     3
                // } else {
                //     26 - dependency.len()
                // });
                //
                // format!(
                //     "  {}{}{}",
                //     dependency.bright_black(),
                //     version_pad,
                //     versions.get(0).unwrap().1
                // )

                versions
                    .iter()
                    .map(|(package, version)| {
                        let location = format!(
                            "{} {}",
                            dependency.white(),
                            format!("in {}", package).bright_black()
                        );
                        let len = format!("{} in {}", dependency, package).len();
                        let version_pad = " ".repeat(if len >= 50 { 3 } else { 50 - len });

                        format!(
                            "  {}{}{}",
                            location,
                            version_pad,
                            version.to_string().yellow()
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn why(&self) -> Cow<'static, str> {
        Cow::Owned(format!("Dependencies for {} aren't synced.", self.r#type))
    }

    fn fix(&mut self, _package_type: &super::PackageType) -> anyhow::Result<()> {
        Ok(())
    }
}
