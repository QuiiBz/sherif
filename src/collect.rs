use crate::packages::root::RootPackage;
use crate::packages::semversion::SemVersion;
use crate::packages::{Config, Package, PackagesList};
use crate::printer::print_error;
use crate::rules::multiple_dependency_versions::MultipleDependencyVersionsIssue;
use crate::rules::non_existant_packages::NonExistantPackagesIssue;
use crate::rules::packages_without_package_json::PackagesWithoutPackageJsonIssue;
use crate::rules::types_in_dependencies::TypesInDependenciesIssue;
use crate::rules::unsync_similar_dependencies::{
    SimilarDependency, UnsyncSimilarDependenciesIssue,
};
use crate::rules::{BoxIssue, IssuesList, PackageType};
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::path::{Path, PathBuf};

const PNPM_WORKSPACE: &str = "pnpm-workspace.yaml";

#[derive(Debug, Serialize, Deserialize)]
pub struct PnpmWorkspace {
    pub packages: Vec<String>,
}

pub fn collect_packages(root: &Path) -> Result<PackagesList> {
    let root_package = RootPackage::new(root)?;
    let mut packages = Vec::new();
    let mut packages_list = root_package.get_workspaces();
    let mut excluded_paths = Vec::new();
    let mut non_existant_paths = Vec::new();
    let mut is_pnpm_workspace = false;

    if packages_list.is_none() {
        let pnpm_workspace = root.join(PNPM_WORKSPACE);

        if !pnpm_workspace.is_file() {
            return Err(anyhow!(
                    "No `workspaces` field in the root `package.json`, or `pnpm-workspace.yaml` file not found in {:?}",
                    root
                ));
        }

        let root_package = fs::read_to_string(pnpm_workspace)?;
        let workspace: PnpmWorkspace = serde_yaml::from_str(&root_package)?;

        packages_list = Some(workspace.packages);
        is_pnpm_workspace = true;
    }

    let mut packages_issues: Vec<BoxIssue> = Vec::new();

    let mut add_package = |packages_issues: &mut Vec<BoxIssue>, path: PathBuf| {
        // Ignore hidden directories, e.g. `.npm`, `.react-email`
        if let Some(stem) = path.file_stem() {
            if let Some(stem) = stem.to_str() {
                if stem.starts_with('.') {
                    return;
                }
            }
        }

        match Package::new(path.clone()) {
            Ok(package) => packages.push(package),
            Err(error) => {
                if error.to_string().contains("not found") {
                    packages_issues.push(PackagesWithoutPackageJsonIssue::new(
                        path.to_string_lossy().to_string(),
                    ));
                } else {
                    print_error("Failed to collect package", &error.to_string());
                    std::process::exit(1);
                }
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
                        let directory = root.join(directory);

                        excluded_paths.push(directory.to_string_lossy().to_string());
                    } else {
                        let directory = package.trim_start_matches('!');
                        let directory = root.join(directory);

                        excluded_paths.push(directory.to_string_lossy().to_string());
                    }

                    return false;
                }

                true
            })
            .collect::<Vec<_>>();

        let mut expanded_packages = Vec::new();

        for package in packages {
            if let Some((directory, subdirectory)) = package.split_once("/*/") {
                let directory = root.join(directory);

                match directory.read_dir() {
                    Ok(expanded_folders) => {
                        for expanded_folder in expanded_folders.flatten() {
                            let expanded_folder = expanded_folder.path();

                            if expanded_folder.is_dir() {
                                let path = expanded_folder
                                    .to_string_lossy()
                                    .to_string()
                                    .replace(&(root.to_string_lossy().to_string() + "/"), "")
                                    + "/"
                                    + subdirectory;

                                expanded_packages.push(path);
                            }
                        }
                    }
                    Err(_) => {
                        non_existant_paths.push(package.to_string());
                        continue;
                    }
                }
            } else if let Some((directory, subdirectory)) = package.split_once("/**/") {
                let directory = root.join(directory);

                match directory.read_dir() {
                    Ok(expanded_folders) => {
                        for expanded_folder in expanded_folders.flatten() {
                            let expanded_folder = expanded_folder.path();

                            if expanded_folder.is_dir() {
                                let path = expanded_folder
                                    .to_string_lossy()
                                    .to_string()
                                    .replace(&(root.to_string_lossy().to_string() + "/"), "")
                                    + "/"
                                    + subdirectory;

                                expanded_packages.push(path);
                            }
                        }
                    }
                    Err(_) => {
                        non_existant_paths.push(package.to_string());
                        continue;
                    }
                }
            } else {
                expanded_packages.push(package.to_string());
            }
        }

        for package in &expanded_packages {
            if package.ends_with('*') {
                let directory_match = package.trim_end_matches('*');

                let packages = match directory_match.ends_with('/') {
                    true => {
                        let directory = directory_match.trim_end_matches('/');
                        let directory = root.join(directory);

                        match directory.read_dir() {
                            Ok(packages) => packages.into_iter().collect::<Result<Vec<_>, _>>()?,
                            Err(_) => {
                                non_existant_paths.push(package.to_string());
                                continue;
                            }
                        }
                    }
                    false => {
                        let directory = root.join(directory_match);
                        let directory = directory.parent().unwrap().to_path_buf();

                        match directory.read_dir() {
                            Ok(packages) => packages
                                .into_iter()
                                .filter(|package| {
                                    if let Ok(package) = package {
                                        return package.file_type().unwrap().is_dir()
                                            && package
                                                .file_name()
                                                .to_string_lossy()
                                                .starts_with(directory_match);
                                    }

                                    true
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                            Err(_) => {
                                non_existant_paths.push(package.to_string());
                                continue;
                            }
                        }
                    }
                };

                for package in packages {
                    if package.file_type()?.is_dir() {
                        let path = package.path();
                        let real_path = path.to_string_lossy().to_string();
                        let mut is_excluded = false;

                        for excluded_path in &excluded_paths {
                            if real_path.starts_with(excluded_path)
                                && !real_path.replace(excluded_path, "").contains('/')
                            {
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
                let path = root.join(package);

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

pub fn collect_issues(config: &Config, packages_list: PackagesList) -> IssuesList<'_> {
    let mut issues = IssuesList::new(&config.ignore_rule);

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
    let mut joined_dependencies = IndexMap::new();
    let mut similar_dependencies_by_package = IndexMap::new();

    if let Some(dependencies) = root_package.get_dependencies() {
        joined_dependencies.extend(dependencies);
    }

    if let Some(dev_dependencies) = root_package.get_dev_dependencies() {
        joined_dependencies.extend(dev_dependencies);
    }

    for (name, version) in joined_dependencies {
        if version.is_valid() {
            all_dependencies
                .entry(name)
                .or_insert_with(IndexMap::new)
                .insert(root_package.get_path(), version);
        }
    }

    for package in packages {
        if package.is_ignored(&config.ignore_package) {
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
            if version.is_valid() {
                all_dependencies
                    .entry(name)
                    .or_insert_with(IndexMap::new)
                    .insert(package.get_path(), version);
            }
        }
    }

    for (name, versions) in all_dependencies {
        if let Ok(similar_dependency) = SimilarDependency::try_from(name.as_str()) {
            for (path, version) in versions.iter() {
                similar_dependencies_by_package
                    .entry(path.clone())
                    .or_insert_with(
                        IndexMap::<SimilarDependency, IndexMap<SemVersion, String>>::new,
                    )
                    .entry(similar_dependency.clone())
                    .or_insert_with(IndexMap::new)
                    .insert(version.clone(), name.clone());
            }
        }

        let mut filtered_versions = versions
            .iter()
            .filter(|(_, version)| {
                !config
                    .ignore_dependency
                    .contains(&format!("{}@{}", name, version))
            })
            .map(|(path, version)| (path.clone(), version.clone()))
            .collect::<IndexMap<_, _>>();

        if filtered_versions.len() > 1
            && !filtered_versions
                .values()
                .collect::<Vec<_>>()
                .windows(2)
                .all(|window| window[0] == window[1])
            && !config.ignore_dependency.contains(&name)
            && !config.ignore_dependency.iter().any(|dependency| {
                if dependency.ends_with('*') {
                    if dependency.starts_with('*') {
                        return name
                            .contains(dependency.trim_start_matches('*').trim_end_matches('*'));
                    }
                    return name.starts_with(dependency.trim_end_matches('*'));
                } else if dependency.starts_with('*') {
                    return name.ends_with(dependency.trim_start_matches('*'));
                }
                false
            })
        {
            filtered_versions.sort_keys();

            issues.add_raw(
                PackageType::None,
                MultipleDependencyVersionsIssue::new(
                    name,
                    filtered_versions,
                    config.select.clone(),
                ),
            );
        }
    }

    for (path, similar_dependencies) in similar_dependencies_by_package {
        for (similar_dependency, versions) in similar_dependencies {
            if versions.len() > 1 {
                issues.add_raw(
                    PackageType::Package(path.clone()),
                    UnsyncSimilarDependenciesIssue::new(similar_dependency, versions),
                );
            }
        }
    }

    issues
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::args::Args;
    use debugless_unwrap::DebuglessUnwrapErr;

    #[test]
    fn collect_packages_unknown_dir() {
        let root = Path::new("unknown");
        let result = collect_packages(root);

        assert!(result.is_err());
        assert_eq!(
            result.debugless_unwrap_err().to_string(),
            "Path \"unknown\" is not a directory"
        );
    }

    #[test]
    fn collect_packages_empty_dir() {
        let root = Path::new("fixtures/empty");
        let result = collect_packages(root);

        assert!(result.is_err());
        assert_eq!(
            result.debugless_unwrap_err().to_string(),
            "`package.json` not found in \"fixtures/empty\""
        );
    }

    #[test]
    fn collect_packages_basic() {
        let root = Path::new("fixtures/basic");
        let result = collect_packages(root);

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
        let root = Path::new("fixtures/pnpm");
        let result = collect_packages(root);

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
    fn collect_packages_yarn_nohoist() {
        let root = Path::new("fixtures/yarn-nohoist");
        let result = collect_packages(root);

        assert!(result.is_ok());
        let PackagesList {
            root_package,
            packages,
            packages_issues,
        } = result.unwrap();

        assert_eq!(root_package.get_name(), "yarn-nohoist");
        assert_eq!(packages.len(), 3);
        assert_eq!(packages_issues.len(), 1);
        assert!(packages_issues[0].name() == "non-existant-packages");
    }

    #[test]
    fn collect_packages_no_workspace_pnpm() {
        let root = Path::new("fixtures/no-workspace-pnpm");
        let result = collect_packages(root);

        assert!(result.is_err());
        assert_eq!(
            result.debugless_unwrap_err().to_string(),
            "No `workspaces` field in the root `package.json`, or `pnpm-workspace.yaml` file not found in \"fixtures/no-workspace-pnpm\""
        );
    }

    #[test]
    fn collect_packages_without_package_json() {
        let root = Path::new("fixtures/without-package-json");
        let result = collect_packages(root);

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
        let root = Path::new("fixtures/ignore-paths");
        let result = collect_packages(root);

        assert!(result.is_ok());
        let PackagesList {
            root_package,
            packages,
            ..
        } = result.unwrap();

        assert_eq!(root_package.get_name(), "ignore-paths");
        assert_eq!(packages.len(), 4);

        let mut packages = packages
            .into_iter()
            .map(|package| package.get_name().clone().unwrap().to_string())
            .collect::<Vec<_>>();
        packages.sort();

        assert_eq!(packages[0], "d");
        assert_eq!(packages[1], "docs");
        assert_eq!(packages[2], "e");
        assert_eq!(packages[3], "ghi");
    }

    #[test]
    fn collect_root_issues() {
        let args = Args {
            path: "fixtures/root-issues".into(),
            fix: false,
            select: None,
            no_install: true,
            fail_on_warnings: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args.path).unwrap();
        let config = args.into();
        assert_eq!(packages_list.root_package.get_name(), "root-issues");

        let issues = collect_issues(&config, packages_list);
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
            select: None,
            no_install: true,
            fail_on_warnings: false,
            path: "fixtures/root-issues-fixed".into(),
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args.path).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "root-issues-fixed");

        let config = args.into();
        let issues = collect_issues(&config, packages_list);
        assert_eq!(issues.total_len(), 0);
    }

    #[test]
    fn collect_dependencies() {
        let args = Args {
            path: "fixtures/dependencies".into(),
            fix: false,
            select: None,
            no_install: true,
            fail_on_warnings: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args.path).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "dependencies");

        let config = args.into();
        let issues = collect_issues(&config, packages_list);
        assert_eq!(issues.total_len(), 4);

        let issues = issues.into_iter().collect::<IndexMap<_, _>>();

        assert_eq!(
            issues.get(&PackageType::None).unwrap()[0].name(),
            "multiple-dependency-versions"
        );
        assert_eq!(
            issues.get(&PackageType::None).unwrap()[1].name(),
            "multiple-dependency-versions"
        );
        assert_eq!(
            issues.get(&PackageType::None).unwrap()[2].name(),
            "multiple-dependency-versions"
        );
        assert_eq!(
            issues.get(&PackageType::None).unwrap()[3].name(),
            "multiple-dependency-versions"
        );
    }

    #[test]
    fn collect_dependencies_allow() {
        let args = Args {
            path: "fixtures/dependencies".into(),
            fix: false,
            select: None,
            no_install: false,
            fail_on_warnings: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: vec!["next@4.5.6".to_string(), "*eslint*".to_string()],
        };

        let packages_list = collect_packages(&args.path).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "dependencies");

        let config = args.into();
        let issues = collect_issues(&config, packages_list);
        assert_eq!(issues.total_len(), 1);

        let issues = issues.into_iter().collect::<IndexMap<_, _>>();

        assert_eq!(
            issues.get(&PackageType::None).unwrap()[0].name(),
            "multiple-dependency-versions"
        );
    }

    #[test]
    fn collect_dependencies_without_star() {
        let args = Args {
            path: "fixtures/dependencies-star".into(),
            fix: false,
            select: None,
            no_install: true,
            fail_on_warnings: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args.path).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "dependencies-star");

        let config = args.into();
        let issues = collect_issues(&config, packages_list);
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
    fn collect_dependencies_nested_star() {
        let args = Args {
            path: "fixtures/dependencies-nested-star".into(),
            fix: false,
            select: None,
            no_install: false,
            fail_on_warnings: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args.path).unwrap();
        assert_eq!(
            packages_list.root_package.get_name(),
            "dependencies-nested-star"
        );

        let config = args.into();
        let issues = collect_issues(&config, packages_list);
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
    fn collect_pnpm_glob() {
        let args = Args {
            path: "fixtures/pnpm-glob".into(),
            fix: false,
            select: None,
            no_install: true,
            fail_on_warnings: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args.path).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "pnpm-glob");
        assert_eq!(packages_list.packages.len(), 2);

        let config = args.into();
        let issues = collect_issues(&config, packages_list);
        assert_eq!(issues.total_len(), 0);
    }

    #[test]
    fn collect_unordered_dependencies() {
        let args = Args {
            path: "fixtures/unordered".into(),
            fix: false,
            select: None,
            no_install: false,
            fail_on_warnings: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args.path).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "unordered");
        assert_eq!(packages_list.packages.len(), 1);

        let config = args.into();
        let issues = collect_issues(&config, packages_list);
        assert_eq!(issues.total_len(), 2);

        let issues = issues.into_iter().collect::<IndexMap<_, _>>();

        assert_eq!(
            issues.get(&PackageType::Root).unwrap()[0].name(),
            "unordered-dependencies"
        );
        assert_eq!(
            issues
                .get(&PackageType::Package("fixtures/unordered/docs".to_string()))
                .unwrap()[0]
                .name(),
            "unordered-dependencies"
        );
    }

    #[test]
    fn collect_unsync_similar_dependencies() {
        let args = Args {
            path: "fixtures/unsync".into(),
            fix: false,
            select: None,
            no_install: false,
            fail_on_warnings: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let packages_list = collect_packages(&args.path).unwrap();
        assert_eq!(packages_list.root_package.get_name(), "unsync");
        assert_eq!(packages_list.packages.len(), 2);

        let config = args.into();
        let issues = collect_issues(&config, packages_list);
        assert_eq!(issues.total_len(), 2);

        let issues = issues.into_iter().collect::<IndexMap<_, _>>();

        assert_eq!(
            issues
                .get(&PackageType::Package(
                    "fixtures/unsync/packages/def".to_string()
                ))
                .unwrap()[0]
                .name(),
            "unsync-similar-dependencies"
        );
    }
}
