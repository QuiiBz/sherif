use self::semversion::SemVersion;
use crate::rules::{
    empty_dependencies::{DependencyKind, EmptyDependenciesIssue},
    unordered_dependencies::UnorderedDependenciesIssue,
    BoxIssue,
};
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use root::RootPackage;
use serde::Deserialize;
use std::{fs, path::PathBuf};

pub mod root;
pub mod semversion;

pub struct PackagesList {
    pub root_package: RootPackage,
    pub packages: Vec<Package>,
    pub packages_issues: Vec<BoxIssue>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Workspaces {
    Default(Vec<String>),
    /// https://classic.yarnpkg.com/blog/2018/02/15/nohoist
    Yarn {
        packages: Vec<String>,
    },
}

#[derive(Deserialize, Debug)]
struct PackageInner {
    name: Option<String>,
    private: Option<bool>,
    workspaces: Option<Workspaces>,
    #[serde(rename = "packageManager")]
    package_manager: Option<String>,
    dependencies: Option<IndexMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<IndexMap<String, String>>,
    #[serde(rename = "peerDependencies")]
    peer_dependencies: Option<IndexMap<String, String>>,
    #[serde(rename = "optionalDependencies")]
    optional_dependencies: Option<IndexMap<String, String>>,
}

#[derive(Debug)]
pub struct Package {
    path: PathBuf,
    inner: PackageInner,
}

impl Package {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.is_dir() {
            return Err(anyhow!("Path {:?} is not a directory", path));
        }

        let package_path = path.join("package.json");

        if !package_path.is_file() {
            return Err(anyhow!("`package.json` not found in {:?}", path));
        }

        let root_package = fs::read_to_string(&package_path)?;
        let package: PackageInner = match serde_json::from_str(&root_package) {
            Ok(package) => package,
            Err(err) => return Err(anyhow!("Error while parsing {:?}: {}", package_path, err)),
        };

        Ok(Self {
            path,
            inner: package,
        })
    }

    pub fn get_name(&self) -> &Option<String> {
        &self.inner.name
    }

    pub fn get_path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    pub fn is_private(&self) -> bool {
        self.inner.private.unwrap_or(false)
    }

    fn check_deps(
        &self,
        deps: &Option<IndexMap<String, String>>,
        dependency_kind: DependencyKind,
    ) -> Option<BoxIssue> {
        if let Some(dependencies) = deps {
            if dependencies.is_empty() {
                return Some(EmptyDependenciesIssue::new(dependency_kind));
            }

            let mut sorted_dependencies = dependencies.clone();
            sorted_dependencies.sort_keys();

            if sorted_dependencies.keys().ne(dependencies.keys()) {
                return Some(UnorderedDependenciesIssue::new(dependency_kind));
            }
        }

        None
    }

    pub fn check_dependencies(&self) -> Option<BoxIssue> {
        self.check_deps(&self.inner.dependencies, DependencyKind::Dependencies)
    }

    pub fn check_dev_dependencies(&self) -> Option<BoxIssue> {
        self.check_deps(
            &self.inner.dev_dependencies,
            DependencyKind::DevDependencies,
        )
    }

    pub fn check_peer_dependencies(&self) -> Option<BoxIssue> {
        self.check_deps(
            &self.inner.peer_dependencies,
            DependencyKind::PeerDependencies,
        )
    }

    pub fn check_optional_dependencies(&self) -> Option<BoxIssue> {
        self.check_deps(
            &self.inner.optional_dependencies,
            DependencyKind::OptionalDependencies,
        )
    }

    fn get_deps(
        &self,
        deps: &Option<IndexMap<String, String>>,
    ) -> Option<IndexMap<String, SemVersion>> {
        if let Some(dependencies) = deps {
            let mut versioned_dependencies =
                IndexMap::<String, SemVersion>::with_capacity(dependencies.len());

            for (name, version) in dependencies {
                if let Ok(version) = SemVersion::parse(version) {
                    versioned_dependencies.insert(name.clone(), version);
                }
            }

            return Some(versioned_dependencies);
        }

        None
    }

    pub fn get_dependencies(&self) -> Option<IndexMap<String, SemVersion>> {
        self.get_deps(&self.inner.dependencies)
    }

    pub fn get_dev_dependencies(&self) -> Option<IndexMap<String, SemVersion>> {
        self.get_deps(&self.inner.dev_dependencies)
    }

    pub fn is_ignored(&self, ignored_packages: &[String]) -> bool {
        match self.get_name() {
            Some(name) => ignored_packages.iter().any(|ignored_package| {
                match ignored_package.ends_with('*') {
                    true => {
                        let ignored_package = ignored_package.trim_end_matches('*');

                        name.starts_with(ignored_package)
                            || self.get_path().starts_with(ignored_package)
                    }
                    false => ignored_package == name || ignored_package == &self.get_path(),
                }
            }),
            None => false,
        }
    }
}
