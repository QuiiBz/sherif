use crate::packages::root::RootPackage;
use crate::packages::{Package, PackagesList};
use crate::rules::mutiple_dependency_versions::MultipleDependencyVersionsIssue;
use crate::rules::packages_without_package_json::PackagesWithoutPackageJsonIssue;
use crate::rules::types_in_dependencies::TypesInDependenciesIssue;
use crate::rules::{BoxIssue, IssuesList, PackageType};
use crate::{args::Args, rules::packages_without_package_json};
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde::Deserialize;
use std::path::PathBuf;

const PNPM_WORKSPACE: &str = "pnpm-workspace.yaml";

#[derive(Debug, Deserialize)]
struct PnpmWorkspace {
    packages: Vec<String>,
}

pub fn collect_packages(args: &Args) -> Result<PackagesList> {
    let root_package = RootPackage::new(&args.path)?;
    let mut packages = Vec::new();
    let mut packages_list = root_package.get_workspaces();

    if packages_list.is_none() {
        let pnpm_workspace = args.path.join(PNPM_WORKSPACE);

        if !pnpm_workspace.is_file() {
            return Err(anyhow!(
                    "No `workspaces` field in the root `package.json`, or `pnpm-workspace.yaml` file not found in {:?}",
                    args.path
                ));
        }

        let root_package = std::fs::read_to_string(pnpm_workspace)?;
        let workspace: PnpmWorkspace = serde_yaml::from_str(&root_package)?;

        packages_list = Some(workspace.packages);
    }

    let mut packages_issues: Vec<BoxIssue> = Vec::new();

    let mut add_package = |path: PathBuf| match Package::new(path.clone()) {
        Ok(package) => packages.push(package),
        Err(error) => {
            if error.to_string().contains("package.json") {
                packages_issues.push(PackagesWithoutPackageJsonIssue::new(
                    path.to_string_lossy().to_string(),
                ));
            }
        }
    };

    if let Some(packages) = packages_list {
        for package in packages {
            if package.ends_with('*') {
                let directory = package.trim_end_matches('*').trim_end_matches('/');
                let directory = args.path.join(directory);

                let packages = match directory.read_dir() {
                    Ok(packages) => packages,
                    Err(error) => {
                        return Err(anyhow!("Error while reading {:?}: {}", directory, error))
                    }
                };

                for package in packages {
                    let package = package?;

                    if package.file_type()?.is_dir() {
                        add_package(package.path());
                    }
                }
            } else {
                add_package(args.path.join(package));
            }
        }
    }

    Ok(PackagesList {
        root_package,
        packages,
        packages_issues,
    })
}

pub fn collect_issues(args: &Args, packages_list: PackagesList) -> IssuesList<'_> {
    let mut issues = IssuesList::new(&args.ignore_rule);

    let PackagesList {
        root_package,
        packages,
        packages_issues,
    } = packages_list;

    for package_issue in packages_issues {
        issues.add_raw(PackageType::None, package_issue);
    }

    issues.add(PackageType::Root, root_package.check_private());
    issues.add(PackageType::Root, root_package.check_package_manager());
    issues.add(PackageType::Root, root_package.check_dependencies());
    issues.add(PackageType::Root, root_package.check_dev_dependencies());
    issues.add(PackageType::Root, root_package.check_peer_dependencies());
    issues.add(
        PackageType::Root,
        root_package.check_optional_dependencies(),
    );

    let mut all_dependencies = IndexMap::new();

    for package in packages {
        if args.ignore_package.contains(package.get_name()) {
            continue;
        }

        let package_type = PackageType::Package(package.get_path());

        issues.add(package_type.clone(), package.check_dependencies());
        issues.add(package_type.clone(), package.check_dev_dependencies());
        issues.add(package_type.clone(), package.check_peer_dependencies());
        issues.add(package_type.clone(), package.check_optional_dependencies());

        if let Some(mut dependencies) = package.get_dependencies() {
            if package.is_private() {
                let types_in_dependencies = dependencies
                    .iter()
                    .filter(|(name, _)| name.starts_with("@types/"))
                    .map(|(name, _)| name.to_string())
                    .collect::<Vec<_>>();

                if !types_in_dependencies.is_empty() {
                    issues.add_raw(
                        package_type.clone(),
                        TypesInDependenciesIssue::new(package.get_path(), types_in_dependencies),
                    );
                }
            }

            if let Some(dev_dependencies) = package.get_dev_dependencies() {
                dependencies.extend(dev_dependencies);
            }

            for (name, version) in dependencies {
                if !version.comparators.is_empty() {
                    all_dependencies
                        .entry(name)
                        .or_insert_with(IndexMap::new)
                        .insert(package.get_path(), version);
                }
            }
        }
    }

    for (name, mut versions) in all_dependencies {
        if versions.len() > 1
            && !versions
                .values()
                .collect::<Vec<_>>()
                .windows(2)
                .all(|window| window[0] == window[1])
        {
            let ignored = args.ignore_dependency.contains(&name);

            if !ignored {
                versions.sort_keys();

                issues.add_raw(
                    PackageType::None,
                    MultipleDependencyVersionsIssue::new(name, versions),
                );
            }
        }
    }

    issues
}

#[cfg(test)]
mod test {
    use super::*;
    use debugless_unwrap::DebuglessUnwrapErr;

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
            result.debugless_unwrap_err().to_string(),
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
            result.debugless_unwrap_err().to_string(),
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
        let PackagesList {
            root_package,
            packages,
            packages_issues,
        } = result.unwrap();

        assert_eq!(root_package.get_name(), "basic");
        assert_eq!(packages.len(), 3);
        assert_eq!(packages_issues.len(), 0);
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
        let PackagesList {
            root_package,
            packages,
            packages_issues,
        } = result.unwrap();

        assert_eq!(root_package.get_name(), "pnpm");
        assert_eq!(packages.len(), 3);
        assert_eq!(packages_issues.len(), 0);
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
            result.debugless_unwrap_err().to_string(),
            "No `workspaces` field in the root `package.json`, or `pnpm-workspace.yaml` file not found in \"fixtures/no-workspace-pnpm\""
        );
    }

    #[test]
    fn collect_packages_without_package_json() {
        let args = Args {
            path: "fixtures/without-package-json".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let result = collect_packages(&args);

        assert!(result.is_ok());
        let PackagesList {
            root_package,
            packages,
            packages_issues,
        } = result.unwrap();

        assert_eq!(root_package.get_name(), "without-package-json");
        assert_eq!(packages.len(), 1);
        assert_eq!(packages_issues.len(), 1);

        colored::control::set_override(false);
        assert_eq!(
            packages_issues[0].message(),
            "2 packages doesn't have a package.json file: fixtures/without-package-json/packages/abc, fixtures/without-package-json/docs"
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

        let packages_list = collect_packages(&args).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "root-issues");

        let issues = collect_issues(&args, packages_list);
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

        let packages_list = collect_packages(&args).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "root-issues-fixed");

        let issues = collect_issues(&args, packages_list);
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

        let packages_list = collect_packages(&args).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "dependencies");

        colored::control::set_override(false);
        let issues = collect_issues(&args, packages_list);
        let issues = issues.into_iter().collect::<Vec<_>>();

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].message(), "The `next` dependency has multiple versions, ^1.2.3 being the lowest and ^4.5.6 the highest.".to_string());
        assert_eq!(issues[1].message(), "The `react` dependency has multiple versions, ^1.2.3 being the lowest and ^4.5.6 the highest.".to_string());
    }

    #[test]
    fn collect_dependencies_without_star() {
        let args = Args {
            path: "fixtures/dependencies-star".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "dependencies-star");

        colored::control::set_override(false);
        let issues = collect_issues(&args, packages_list);
        let issues = issues.into_iter().collect::<Vec<_>>();

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].message(), "The `next` dependency has multiple versions, ^1.2.3 being the lowest and ^4.5.6 the highest.".to_string());
    }
}
