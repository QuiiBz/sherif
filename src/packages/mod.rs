use crate::rules::{
    empty_dependencies::{DependencyKind, EmptyDependenciesIssue},
    Issue,
};
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use semver::VersionReq;
use serde::Deserialize;
use std::path::PathBuf;

pub mod root;

#[derive(Deserialize, Debug)]
struct PackageInner {
    name: String,
    private: Option<bool>,
    workspaces: Option<Vec<String>>,
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

        let root_package = path.join("package.json");

        if !root_package.is_file() {
            return Err(anyhow!("`package.json` not found in {:?}", path));
        }

        let root_package = std::fs::read_to_string(root_package)?;
        let package: PackageInner = serde_json::from_str(&root_package)?;

        Ok(Self {
            path,
            inner: package,
        })
    }

    pub fn get_name(&self) -> &String {
        &self.inner.name
    }

    fn check_deps(
        &self,
        deps: &Option<IndexMap<String, String>>,
        dependency_kind: DependencyKind,
    ) -> Option<Box<dyn Issue>> {
        if let Some(dependencies) = deps {
            if dependencies.is_empty() {
                return Some(EmptyDependenciesIssue::new(
                    self.path.to_string_lossy().to_string(),
                    dependency_kind,
                ));
            }
        }

        None
    }

    pub fn check_dependencies(&self) -> Option<Box<dyn Issue>> {
        self.check_deps(&self.inner.dependencies, DependencyKind::Dependencies)
    }

    pub fn check_dev_dependencies(&self) -> Option<Box<dyn Issue>> {
        self.check_deps(
            &self.inner.dev_dependencies,
            DependencyKind::DevDependencies,
        )
    }

    pub fn check_peer_dependencies(&self) -> Option<Box<dyn Issue>> {
        self.check_deps(
            &self.inner.peer_dependencies,
            DependencyKind::PeerDependencies,
        )
    }

    pub fn check_optional_dependencies(&self) -> Option<Box<dyn Issue>> {
        self.check_deps(
            &self.inner.optional_dependencies,
            DependencyKind::OptionalDependencies,
        )
    }

    fn get_deps(
        &self,
        deps: &Option<IndexMap<String, String>>,
    ) -> Option<IndexMap<String, VersionReq>> {
        if let Some(dependencies) = deps {
            let mut versioned_dependencies =
                IndexMap::<String, VersionReq>::with_capacity(dependencies.len());

            for (name, version) in dependencies {
                if let Ok(version) = VersionReq::parse(version) {
                    versioned_dependencies.insert(name.clone(), version);
                }
            }

            return Some(versioned_dependencies);
        }

        None
    }

    pub fn get_dependencies(&self) -> Option<IndexMap<String, VersionReq>> {
        self.get_deps(&self.inner.dependencies)
    }

    pub fn get_dev_dependencies(&self) -> Option<IndexMap<String, VersionReq>> {
        self.get_deps(&self.inner.dev_dependencies)
    }
}
