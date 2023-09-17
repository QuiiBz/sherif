use crate::args::Args;
use crate::packages::root::RootPackage;
use crate::packages::Package;
use crate::rules::mutiple_dependency_versions::MultipleDependencyVersionsIssue;
use crate::rules::IssuesList;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde::Deserialize;
use std::path::Path;

const PNPM_WORKSPACE: &str = "pnpm-workspace.yaml";

#[derive(Debug, Deserialize)]
struct PnpmWorkspace {
    packages: Vec<String>,
}

fn resolve_workspace_packages(
    path: &Path,
    package_root_workspaces: Option<Vec<String>>,
) -> Result<Vec<Package>> {
    let mut all_packages = Vec::new();
    let mut packages_list = package_root_workspaces;

    if packages_list.is_none() {
        let pnpm_workspace = path.join(PNPM_WORKSPACE);

        if !pnpm_workspace.is_file() {
            return Err(anyhow!(
                    "No `workspaces` field in the root `package.json`, or `pnpm-workspace.yaml` file not found in {:?}",
                    path
                ));
        }

        let root_package = std::fs::read_to_string(pnpm_workspace)?;
        let workspace: PnpmWorkspace = serde_yaml::from_str(&root_package)?;

        packages_list = Some(workspace.packages);
    }

    if let Some(packages) = packages_list {
        for package in packages {
            if package.ends_with('*') {
                let directory = package.trim_end_matches('*').trim_end_matches('/');
                let packages = path.join(directory).read_dir()?;

                for package in packages {
                    if let Ok(package) = Package::new(package?.path()) {
                        all_packages.push(package);
                    }
                }
            } else {
                let package = path.join(package);

                if let Ok(package) = Package::new(package) {
                    all_packages.push(package);
                }
            }
        }
    }

    Ok(all_packages)
}

pub fn collect_packages(args: &Args) -> Result<(RootPackage, Vec<Package>)> {
    let root_package = RootPackage::new(&args.path)?;
    let packages = resolve_workspace_packages(&args.path, root_package.get_workspaces())?;

    Ok((root_package, packages))
}

pub fn collect_issues<'a>(
    args: &'a Args,
    root_package: &RootPackage,
    packages: &[Package],
) -> IssuesList<'a> {
    let mut issues = IssuesList::new(&args.ignore_rule);

    issues.add(root_package.check_private());
    issues.add(root_package.check_package_manager());
    issues.add(root_package.check_dependencies());
    issues.add(root_package.check_dev_dependencies());
    issues.add(root_package.check_peer_dependencies());
    issues.add(root_package.check_optional_dependencies());

    let mut all_dependencies = IndexMap::new();

    for package in packages {
        if args.ignore_package.contains(package.get_name()) {
            continue;
        }

        issues.add(package.check_dependencies());
        issues.add(package.check_dev_dependencies());
        issues.add(package.check_peer_dependencies());
        issues.add(package.check_optional_dependencies());

        if let Some(mut dependencies) = package.get_dependencies() {
            if let Some(dev_dependencies) = package.get_dev_dependencies() {
                dependencies.extend(dev_dependencies);
            }

            for (name, version) in dependencies {
                all_dependencies
                    .entry(name)
                    .or_insert_with(Vec::new)
                    .push(version);
            }
        }
    }

    for (name, versions) in all_dependencies {
        if versions.len() > 1 && !versions.windows(2).all(|window| window[0] == window[1]) {
            let ignored = args.ignore_dependency.contains(&name);

            issues.add_raw(MultipleDependencyVersionsIssue::new(
                name, versions, ignored,
            ));
        }
    }

    issues
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn collect_packages_unknown_dir() {
        let args = Args {
            path: "unknown".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let result = collect_packages(&args);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Path \"unknown\" is not a directory"
        );
    }

    #[test]
    fn collect_packages_empty_dir() {
        let args = Args {
            path: "fixtures/empty".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let result = collect_packages(&args);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "`package.json` not found in \"fixtures/empty\""
        );
    }

    #[test]
    fn collect_packages_basic() {
        let args = Args {
            path: "fixtures/basic".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let result = collect_packages(&args);

        assert!(result.is_ok());
        let (root_package, packages) = result.unwrap();

        assert_eq!(root_package.get_name(), "basic");
        assert_eq!(packages.len(), 3);
    }

    #[test]
    fn collect_packages_pnpm() {
        let args = Args {
            path: "fixtures/pnpm".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let result = collect_packages(&args);

        assert!(result.is_ok());
        let (root_package, packages) = result.unwrap();

        assert_eq!(root_package.get_name(), "pnpm");
        assert_eq!(packages.len(), 3);
    }

    #[test]
    fn collect_packages_no_workspace_pnpm() {
        let args = Args {
            path: "fixtures/no-workspace-pnpm".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let result = collect_packages(&args);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No `workspaces` field in the root `package.json`, or `pnpm-workspace.yaml` file not found in \"fixtures/no-workspace-pnpm\""
        );
    }

    #[test]
    fn collect_root_issues() {
        let args = Args {
            path: "fixtures/root-issues".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let (root_package, packages) = collect_packages(&args).unwrap();
        assert_eq!(root_package.get_name(), "root-issues");

        let issues = collect_issues(&args, &root_package, &packages);
        let issues = issues.into_iter().collect::<Vec<_>>();

        assert_eq!(issues.len(), 4);
        assert_eq!(issues[0].name(), "root-package-private-field");
        assert_eq!(issues[1].name(), "root-package-manager-field");
        assert_eq!(issues[2].name(), "root-package-dependencies");
        assert_eq!(issues[3].name(), "empty-dependencies");
    }

    #[test]
    fn collect_root_issues_fixed() {
        let args = Args {
            path: "fixtures/root-issues-fixed".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let (root_package, packages) = collect_packages(&args).unwrap();
        assert_eq!(root_package.get_name(), "root-issues-fixed");

        let issues = collect_issues(&args, &root_package, &packages);
        let issues = issues.into_iter().collect::<Vec<_>>();

        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn collect_dependencies() {
        let args = Args {
            path: "fixtures/dependencies".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let (root_package, packages) = collect_packages(&args).unwrap();
        assert_eq!(root_package.get_name(), "dependencies");

        colored::control::set_override(false);
        let issues = collect_issues(&args, &root_package, &packages);
        let issues = issues.into_iter().collect::<Vec<_>>();

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].message(), "The `next` dependency has multiple versions, ^1.2.3 being the lowest and ^4.5.6 the highest.".to_string());
        assert_eq!(issues[1].message(), "The `react` dependency has multiple versions, ^1.2.3 being the lowest and ^4.5.6 the highest.".to_string());
    }
}
