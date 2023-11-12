use super::Package;
use crate::rules::{
    root_package_dependencies::RootPackageDependenciesIssue,
    root_package_manager_field::RootPackageManagerFieldIssue,
    root_package_private_field::RootPackagePrivateFieldIssue, BoxIssue,
};
use anyhow::Result;
use std::path::Path;

#[derive(Debug)]
pub struct RootPackage(Package);

impl RootPackage {
    pub fn new(path: &Path) -> Result<Self> {
        let package = Package::new(path.to_path_buf())?;

        Ok(Self(package))
    }

    #[cfg(test)]
    pub fn get_name(&self) -> String {
        self.0.get_name().clone().unwrap_or_default()
    }

    pub fn get_workspaces(&self) -> Option<Vec<String>> {
        self.0.inner.workspaces.clone()
    }

    pub fn check_private(&self) -> Option<BoxIssue> {
        match self.0.inner.private {
            Some(true) => None,
            _ => Some(RootPackagePrivateFieldIssue::new()),
        }
    }

    pub fn check_package_manager(&self) -> Option<BoxIssue> {
        match self.0.inner.private.is_none() {
            true => Some(RootPackageManagerFieldIssue::new()),
            false => None,
        }
    }

    pub fn check_dependencies(&self) -> Option<BoxIssue> {
        match self.0.inner.dependencies.is_some() {
            true => Some(RootPackageDependenciesIssue::new()),
            false => None,
        }
    }

    pub fn check_dev_dependencies(&self) -> Option<BoxIssue> {
        self.0.check_dev_dependencies()
    }

    pub fn check_peer_dependencies(&self) -> Option<BoxIssue> {
        self.0.check_peer_dependencies()
    }

    pub fn check_optional_dependencies(&self) -> Option<BoxIssue> {
        self.0.check_optional_dependencies()
    }
}
