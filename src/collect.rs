use crate::args::Args;
use crate::packages::root::RootPackage;
use crate::packages::{Package, PackagesList};
use crate::rules::multiple_dependency_versions::MultipleDependencyVersionsIssue;
use crate::rules::non_existant_packages::NonExistantPackagesIssue;
use crate::rules::packages_without_package_json::PackagesWithoutPackageJsonIssue;
use crate::rules::types_in_dependencies::TypesInDependenciesIssue;
use crate::rules::{BoxIssue, IssuesList, PackageType};
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
    let mut excluded_paths = Vec::new();
    let mut non_existant_paths = Vec::new();
    let mut is_pnpm_workspace = false;

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
        is_pnpm_workspace = true;
    }

    let mut packages_issues: Vec<BoxIssue> = Vec::new();

    let mut add_package =
        |packages_issues: &mut Vec<BoxIssue>, path: PathBuf| match Package::new(path.clone()) {
            Ok(package) => packages.push(package),
            Err(error) => {
                if error.to_string().contains("package.json") {
                    packages_issues.push(PackagesWithoutPackageJsonIssue::new(
                        path.to_string_lossy().to_string(),
                    ));
                }
            }
        };

    if let Some(packages) = &packages_list {
        let packages = packages
            .iter()
            .filter(|package| {
                if package.starts_with('!') {
                    if package.ends_with('*') {
                        let directory = package
                            .trim_start_matches('!')
                            .trim_end_matches('*')
                            .trim_end_matches('/');
                        let directory = args.path.join(directory);

                        excluded_paths.push(directory.to_string_lossy().to_string());
                    } else {
                        let directory = package.trim_start_matches('!');
                        let directory = args.path.join(directory);

                        excluded_paths.push(directory.to_string_lossy().to_string());
                    }

                    return false;
                }

                true
            })
            .collect::<Vec<_>>();

        for package in &packages {
            if package.ends_with('*') {
                let directory = package.trim_end_matches('*').trim_end_matches('/');
                let directory = args.path.join(directory);

                let packages = match directory.read_dir() {
                    Ok(packages) => packages,
                    Err(_) => {
                        non_existant_paths.push(package.to_string());
                        continue;
                    }
                };

                for package in packages {
                    let package = package?;

                    if package.file_type()?.is_dir() {
                        let path = package.path();
                        let real_path = path.to_string_lossy().to_string();
                        let mut is_excluded = false;

                        for excluded_path in &excluded_paths {
                            if real_path.starts_with(excluded_path) {
                                is_excluded = true;
                                break;
                            }
                        }

                        if !is_excluded {
                            add_package(&mut packages_issues, path);
                        }
                    }
                }
            } else {
                let path = args.path.join(package);

                match path.is_dir() {
                    true => add_package(&mut packages_issues, path),
                    false => non_existant_paths.push(package.to_string()),
                }
            }
        }

        if !non_existant_paths.is_empty() {
            packages_issues.push(NonExistantPackagesIssue::new(
                is_pnpm_workspace,
                packages_list.unwrap(),
                non_existant_paths,
            ));
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

        let mut joined_dependencies = IndexMap::new();

        if let Some(dependencies) = package.get_dependencies() {
            if package.is_private() {
                let types_in_dependencies = dependencies
                    .iter()
                    .filter(|(name, _)| name.starts_with("@types/"))
                    .map(|(name, _)| name.to_string())
                    .collect::<Vec<_>>();

                if !types_in_dependencies.is_empty() {
                    issues.add_raw(
                        package_type.clone(),
                        TypesInDependenciesIssue::new(types_in_dependencies),
                    );
                }
            }

            joined_dependencies.extend(dependencies);
        }

        if let Some(dev_dependencies) = package.get_dev_dependencies() {
            joined_dependencies.extend(dev_dependencies);
        }

        for (name, version) in joined_dependencies {
            if !version.comparators.is_empty() {
                all_dependencies
                    .entry(name)
                    .or_insert_with(IndexMap::new)
                    .insert(package.get_path(), version);
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
            fix: false,
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
            fix: false,
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
            fix: false,
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
        assert_eq!(packages_issues.len(), 1);
        assert!(packages_issues[0].name() == "non-existant-packages");
    }

    #[test]
    fn collect_packages_pnpm() {
        let args = Args {
            path: "fixtures/pnpm".into(),
            fix: false,
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
        assert_eq!(packages_issues.len(), 1);
        assert!(packages_issues[0].name() == "non-existant-packages");
    }

    #[test]
    fn collect_packages_no_workspace_pnpm() {
        let args = Args {
            path: "fixtures/no-workspace-pnpm".into(),
            fix: false,
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
            fix: false,
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
        assert_eq!(packages_issues.len(), 2);
        assert_eq!(packages_issues[0].name(), "packages-without-package-json");
        assert_eq!(packages_issues[1].name(), "packages-without-package-json");
    }

    #[test]
    fn collect_packages_ignore_paths() {
        let args = Args {
            path: "fixtures/ignore-paths".into(),
            fix: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let result = collect_packages(&args);

        assert!(result.is_ok());
        let PackagesList {
            root_package,
            packages,
            ..
        } = result.unwrap();

        assert_eq!(root_package.get_name(), "ignore-paths");
        assert_eq!(packages.len(), 2);

        let mut packages = packages
            .into_iter()
            .map(|package| package.get_name().to_string())
            .collect::<Vec<_>>();
        packages.sort();

        assert_eq!(packages[0], "docs");
        assert_eq!(packages[1], "ghi");
    }

    #[test]
    fn collect_root_issues() {
        let args = Args {
            path: "fixtures/root-issues".into(),
            fix: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "root-issues");

        let issues = collect_issues(&args, packages_list);
        assert_eq!(issues.total_len(), 4);

        let issues = issues.into_iter().collect::<IndexMap<_, _>>();
        assert_eq!(
            issues.get(&PackageType::Root).unwrap()[0].name(),
            "root-package-private-field"
        );
        assert_eq!(
            issues.get(&PackageType::Root).unwrap()[1].name(),
            "root-package-manager-field"
        );
        assert_eq!(
            issues.get(&PackageType::Root).unwrap()[2].name(),
            "root-package-dependencies"
        );
        assert_eq!(
            issues.get(&PackageType::Root).unwrap()[3].name(),
            "empty-dependencies"
        );
    }

    #[test]
    fn collect_root_issues_fixed() {
        let args = Args {
            fix: false,
            path: "fixtures/root-issues-fixed".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "root-issues-fixed");

        let issues = collect_issues(&args, packages_list);
        assert_eq!(issues.total_len(), 0);
    }

    #[test]
    fn collect_dependencies() {
        let args = Args {
            path: "fixtures/dependencies".into(),
            fix: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "dependencies");

        let issues = collect_issues(&args, packages_list);
        assert_eq!(issues.total_len(), 2);

        let issues = issues.into_iter().collect::<IndexMap<_, _>>();

        assert_eq!(
            issues.get(&PackageType::None).unwrap()[0].name(),
            "multiple-dependency-versions"
        );
        assert_eq!(
            issues.get(&PackageType::None).unwrap()[1].name(),
            "multiple-dependency-versions"
        );
    }

    #[test]
    fn collect_dependencies_without_star() {
        let args = Args {
            path: "fixtures/dependencies-star".into(),
            fix: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "dependencies-star");

        let issues = collect_issues(&args, packages_list);
        assert_eq!(issues.total_len(), 1);

        let issues = issues.into_iter().collect::<IndexMap<_, _>>();

        assert_eq!(
            issues.get(&PackageType::None).unwrap()[0].name(),
            "multiple-dependency-versions"
        );
    }
}
