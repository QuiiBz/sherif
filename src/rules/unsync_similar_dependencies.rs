use super::Issue;
use crate::packages::semversion::SemVersion;
use colored::Colorize;
use indexmap::IndexMap;
use std::{borrow::Cow, fmt::Display, hash::Hash};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum SimilarDependency {
    Trpc,
    React,
    NextJS,
    Storybook,
    Turborepo,
    TanstackQuery,
    Prisma,
    TypescriptEslint,
    EslintStylistic,
    Playwright,
}

impl Display for SimilarDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trpc => write!(f, "tRPC"),
            Self::React => write!(f, "React"),
            Self::NextJS => write!(f, "Next.js"),
            Self::Storybook => write!(f, "Storybook"),
            Self::Turborepo => write!(f, "Turborepo"),
            Self::TanstackQuery => write!(f, "Tanstack Query"),
            Self::Prisma => write!(f, "Prisma"),
            Self::TypescriptEslint => write!(f, "typescript-eslint"),
            Self::EslintStylistic => write!(f, "ESLint Stylistic"),
            Self::Playwright => write!(f, "Playwright"),
        }
    }
}

impl TryFrom<&str> for SimilarDependency {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "@trpc/client" | "@trpc/server" | "@trpc/next" | "@trpc/react-query" => Ok(Self::Trpc),
            "react" | "react-dom" => Ok(Self::React),
            "eslint-config-next"
            | "@next/eslint-plugin-next"
            | "@next/font"
            | "@next/bundle-analyzer"
            | "@next/mdx"
            | "next"
            | "@next/third-parties" => Ok(Self::NextJS),
            "eslint-config-turbo"
            | "eslint-plugin-turbo"
            | "@turbo/gen"
            | "turbo-ignore"
            | "turbo" => Ok(Self::Turborepo),
            "sb"
            | "storybook"
            | "@storybook/codemod"
            | "@storybook/cli"
            | "@storybook/channels"
            | "@storybook/addon-actions"
            | "@storybook/addon-links"
            | "@storybook/react"
            | "@storybook/react-native"
            | "@storybook/components"
            | "@storybook/addon-backgrounds"
            | "@storybook/addon-viewport"
            | "@storybook/angular"
            | "@storybook/addon-a11y"
            | "@storybook/addon-jest"
            | "@storybook/client-logger"
            | "@storybook/node-logger"
            | "@storybook/core"
            | "@storybook/addon-storysource"
            | "@storybook/html"
            | "@storybook/core-events"
            | "@storybook/svelte"
            | "@storybook/ember"
            | "@storybook/addon-ondevice-backgrounds"
            | "@storybook/addon-ondevice-notes"
            | "@storybook/preact"
            | "@storybook/theming"
            | "@storybook/router"
            | "@storybook/addon-docs"
            | "@storybook/addon-ondevice-actions"
            | "@storybook/source-loader"
            | "@storybook/preset-create-react-app"
            | "@storybook/web-components"
            | "@storybook/addon-essentials"
            | "@storybook/server"
            | "@storybook/addon-toolbars"
            | "@storybook/addon-controls"
            | "@storybook/core-common"
            | "@storybook/builder-webpack5"
            | "@storybook/core-server"
            | "@storybook/csf-tools"
            | "@storybook/addon-measure"
            | "@storybook/addon-outline"
            | "@storybook/addon-ondevice-controls"
            | "@storybook/instrumenter"
            | "@storybook/addon-interactions"
            | "@storybook/docs-tools"
            | "@storybook/builder-vite"
            | "@storybook/telemetry"
            | "@storybook/core-webpack"
            | "@storybook/preset-html-webpack"
            | "@storybook/preset-preact-webpack"
            | "@storybook/preset-svelte-webpack"
            | "@storybook/preset-react-webpack"
            | "@storybook/html-webpack5"
            | "@storybook/preact-webpack5"
            | "@storybook/svelte-webpack5"
            | "@storybook/web-components-webpack5"
            | "@storybook/preset-server-webpack"
            | "@storybook/react-webpack5"
            | "@storybook/server-webpack5"
            | "@storybook/addon-highlight"
            | "@storybook/blocks"
            | "@storybook/builder-manager"
            | "@storybook/react-vite"
            | "@storybook/svelte-vite"
            | "@storybook/web-components-vite"
            | "@storybook/nextjs"
            | "@storybook/types"
            | "@storybook/manager"
            | "@storybook/csf-plugin"
            | "@storybook/preview"
            | "@storybook/manager-api"
            | "@storybook/preview-api"
            | "@storybook/html-vite"
            | "@storybook/sveltekit"
            | "@storybook/preact-vite"
            | "@storybook/addon-mdx-gfm"
            | "@storybook/react-dom-shim"
            | "create-storybook"
            | "@storybook/addon-onboarding"
            | "@storybook/react-native-theming"
            | "@storybook/addon-themes"
            | "@storybook/test"
            | "@storybook/react-native-ui"
            | "@storybook/experimental-nextjs-vite"
            | "@storybook/experimental-addon-test"
            | "@storybook/react-native-web-vite" => Ok(Self::Storybook),
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
            "prisma"
            | "@prisma/client"
            | "@prisma/instrumentation"
            | "@prisma/adapter-pg"
            | "@prisma/adapter-neon"
            | "@prisma/adapter-planetscale"
            | "@prisma/adapter-d1"
            | "@prisma/adapter-libsql"
            | "@prisma/adapter-pg-worker"
            | "@prisma/pg-worker" => Ok(Self::Prisma),
            "typescript-eslint"
            | "@typescript-eslint/eslint-plugin"
            | "@typescript-eslint/parser" => Ok(Self::TypescriptEslint),
            "@stylistic/eslint-plugin-js"
            | "@stylistic/eslint-plugin-ts"
            | "@stylistic/eslint-plugin-migrate"
            | "@stylistic/eslint-plugin"
            | "@stylistic/eslint-plugin-jsx"
            | "@stylistic/eslint-plugin-plus" => Ok(Self::EslintStylistic),
            "playwright" | "@playwright/test" => Ok(Self::Playwright),
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
        Cow::Owned(format!(
            "Similar {} dependencies should use the same version.",
            self.r#type
        ))
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
        assert_eq!(
            issue.why(),
            "Similar React dependencies should use the same version."
        );
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
