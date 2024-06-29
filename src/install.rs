use std::{fs, process::Command};

use inquire::{Confirm, Select};
use colored::Colorize;


pub fn ask () {
    let should_run_install = Confirm::new("Do you want to run `install`")
        .with_default(true)
        .prompt();
 
    match should_run_install {
        Ok(true) => {
            install();
        }
        Ok(false) => {
            println!("Don't forget to run `install` manually.");
        }
        Err(_) => {
            println!("Something went wrong. Run `install` manually.");
        }
    }
}

fn install () {
    println!("Running `install`...");

    let package_manager = detect_package_manager();
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

    manual_select_package_manager()
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