use std::{fs, process::Command};

use inquire::Select;
use colored::Colorize;


pub fn run () {
    let mut package_manager = detect_package_manager();

    if package_manager.is_empty() {
        println!("Could not auto-detect package manager.");
       package_manager = manual_select_package_manager();
    }

    println!("Running install using: {}...", package_manager);

    Command::new(package_manager)
        .arg("install")
        .output()
        .expect("Failed to run `install`.");

    println!("{} Install completed.", "✓".green());
}

fn detect_package_manager () -> String {
    if fs::metadata("package-lock.json").is_ok() {
        return "npm".to_string();
    } 
    
    if fs::metadata("yarn.lock").is_ok() {
        return "yarn".to_string();
    } 
    
    if fs::metadata("pnpm-lock.yaml").is_ok() {
        return "pnpm".to_string();
    } 

    return "".to_string();
}

fn manual_select_package_manager () -> String {
    let package_manager = Select::new("Select a package manager", vec!["npm", "yarn", "pnpm"])
        .prompt();

    match package_manager {
        Ok("npm") => {
            return "npm".to_string();
        }
        Ok("yarn") => {
            return "yarn".to_string();
        }
        Ok("pnpm") => {
            return "pnpm".to_string();
        }
        _ => {
            println!("Invalid package manager selected. Exiting...");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]

mod test {
    #[test]
    fn test_detect_package_manager() {
        use super::*;
        use std::fs;

        fs::File::create("package-lock.json").unwrap();
        assert_eq!(detect_package_manager(), "npm");

        fs::remove_file("package-lock.json").unwrap();
        fs::File::create("yarn.lock").unwrap();
        assert_eq!(detect_package_manager(), "yarn");

        fs::remove_file("yarn.lock").unwrap();
        fs::File::create("pnpm-lock.yaml").unwrap();
        assert_eq!(detect_package_manager(), "pnpm");

        fs::remove_file("pnpm-lock.yaml").unwrap();
        assert_eq!(detect_package_manager(), "");
    }
}