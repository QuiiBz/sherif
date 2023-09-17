use super::Package;
use crate::rules::{
    root_package_dependencies::RootPackageDependenciesIssue,
    root_package_manager_field::RootPackageManagerFieldIssue,
    root_package_private_field::RootPackagePrivateFieldIssue, Issue,
};
use anyhow::Result;
use indexmap::IndexMap;
use semver::VersionReq;
use std::path::Path;

#[derive(Debug)]
pub struct RootPackage(Package);

impl RootPackage {
    pub fn new(path: &Path) -> Result<Self> {
        let package = Package::new(path.to_path_buf())?;

        Ok(Self(package))
    }

    #[cfg(test)]
    pub fn get_name(&self) -> &String {
        self.0.get_name()
    }

    pub fn get_workspaces(&self) -> Option<Vec<String>> {
        self.0.inner.workspaces.clone()
    }

    pub fn get_dev_dependencies(&self) -> Option<IndexMap<String, VersionReq>> {
        self.0.get_dev_dependencies()
    }

    pub fn check_private(&self) -> Option<Box<dyn Issue>> {
        match self.0.inner.private {
            Some(true) => None,
            Some(false) => Some(RootPackagePrivateFieldIssue::new(true)),
            None => Some(RootPackagePrivateFieldIssue::new(false)),
        }
    }

    pub fn check_package_manager(&self) -> Option<Box<dyn Issue>> {
        match self.0.inner.private.is_none() {
            true => Some(RootPackageManagerFieldIssue::new()),
            false => None,
        }
    }

    pub fn check_dependencies(&self) -> Option<Box<dyn Issue>> {
        match self.0.inner.dependencies.is_some() {
            true => Some(RootPackageDependenciesIssue::new()),
            false => None,
        }
    }

    pub fn check_dev_dependencies(&self) -> Option<Box<dyn Issue>> {
        self.0.check_dev_dependencies()
    }

    pub fn check_peer_dependencies(&self) -> Option<Box<dyn Issue>> {
        self.0.check_peer_dependencies()
    }

    pub fn check_optional_dependencies(&self) -> Option<Box<dyn Issue>> {
        self.0.check_optional_dependencies()
    }
}
