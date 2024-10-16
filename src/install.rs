use crate::printer::get_render_config;
use anyhow::{anyhow, Result};
use colored::Colorize;
use inquire::Select;
use std::{fmt::Display, fs, process::Command, process::Stdio};

const PACKAGE_MANAGERS: [&str; 4] = ["npm", "yarn", "pnpm", "bun"];

#[derive(Debug, PartialEq)]
enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl PackageManager {
    pub fn resolve() -> Result<Self> {
        if fs::metadata("package-lock.json").is_ok() {
            return Ok(PackageManager::Npm);
        } else if fs::metadata("bun.lockb").is_ok() {
            return Ok(PackageManager::Bun);
        } else if fs::metadata("yarn.lock").is_ok() {
            return Ok(PackageManager::Yarn);
        } else if fs::metadata("pnpm-lock.yaml").is_ok() {
            return Ok(PackageManager::Pnpm);
        } 

        let package_manager =
            Select::new("Select a package manager to use", PACKAGE_MANAGERS.to_vec())
                .with_render_config(get_render_config())
                .with_help_message("Enter to select")
                .prompt();

        match package_manager {
            Ok("npm") => Ok(PackageManager::Npm),
            Ok("yarn") => Ok(PackageManager::Yarn),
            Ok("pnpm") => Ok(PackageManager::Pnpm),
            Ok("bun") => Ok(PackageManager::Bun),
            _ => Err(anyhow!("No package manager selected")),
        }
    }
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageManager::Npm => write!(f, "npm"),
            PackageManager::Yarn => write!(f, "yarn"),
            PackageManager::Pnpm => write!(f, "pnpm"),
            PackageManager::Bun => write!(f, "bun"),
        }
    }
}

pub fn install() -> Result<()> {
    let package_manager = PackageManager::resolve()?;

    println!(
        " {}",
        format!("Note: running install command using {}...", package_manager).bright_black(),
    );
    println!();

    let mut command = Command::new(package_manager.to_string())
        .arg("install")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = command.wait()?;
    if !status.success() {
        return Err(anyhow!("Install command failed"));
    }

    println!();
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{args::Args, collect::collect_packages};
    use serde_json::Value;
    use std::fs;

    #[test]
    fn test_detect_package_manager() {
        use super::*;
        use std::fs;

        fs::File::create("package-lock.json").unwrap();
        assert_eq!(PackageManager::resolve().unwrap(), PackageManager::Npm);

        fs::remove_file("package-lock.json").unwrap();
        fs::File::create("yarn.lock").unwrap();
        assert_eq!(PackageManager::resolve().unwrap(), PackageManager::Yarn);

        fs::remove_file("yarn.lock").unwrap();
        fs::File::create("pnpm-lock.yaml").unwrap();
        assert_eq!(PackageManager::resolve().unwrap(), PackageManager::Pnpm);

        fs::remove_file("pnpm-lock.yaml").unwrap();
    }

    #[test]
    fn test_install_run() {
        let args = Args {
            path: "fixtures/install".into(),
            fix: false,
            no_install: false,
            ignore_rule: Vec::new(),
            ignore_package: Vec::new(),
            ignore_dependency: Vec::new(),
        };

        let _ = collect_packages(&args);

        std::env::set_current_dir("fixtures/install").unwrap();
        super::install().unwrap();

        // Test if the previously empty package-lock.json now contains the "install" name to indicate that the install command was run
        let file = fs::File::open("package-lock.json");
        let json: Result<Value, serde_json::Error> = serde_json::from_reader(file.unwrap());
        assert_eq!(json.unwrap()["name"], "install");

        std::env::set_current_dir("../../").unwrap();
    }
}
