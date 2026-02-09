// File: src/commands/new_project.rs

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use crate::config::SummitConfig;

const MAIN_SM_TEMPLATE: &str = r#"import std::io;

func main(): i8 {
    io::println("Hello, world!");
    ret 0;
}
"#;

/// Creates a new Summit project.
///
/// # Parameters
/// - `project_name`: Optional project name for creating a new folder
///
/// # Returns
/// `Ok(())` if successful, `Err(String)` with error message if failed
pub fn new_project(project_name: Option<String>) -> Result<(), String> {
    match project_name {
        Some(name) => create_project_in_folder(&name),
        None => create_project_in_current_dir(),
    }
}

/// Creates a new project in a specified folder.
///
/// # Parameters
/// - `name`: The project folder name
///
/// # Returns
/// `Ok(())` if successful, `Err(String)` with error message if failed
fn create_project_in_folder(name: &str) -> Result<(), String> {
    let project_path = Path::new(name);

    if project_path.exists() {
        return Err(format!("Directory '{}' already exists", name));
    }

    fs::create_dir(project_path)
        .map_err(|e| format!("Failed to create project directory: {}", e))?;

    let src_path = project_path.join("src");
    fs::create_dir(&src_path)
        .map_err(|e| format!("Failed to create src directory: {}", e))?;

    let main_path = src_path.join("main.sm");
    fs::write(&main_path, MAIN_SM_TEMPLATE)
        .map_err(|e| format!("Failed to create main.sm: {}", e))?;

    let config = SummitConfig::default_config(name);
    let toml_path = project_path.join("Summit.toml");
    config.save(&toml_path)?;

    println!("Created new Summit project: {}", name);
    println!();
    println!("To get started:");
    println!("  cd {}", name);
    println!("  summit run");

    Ok(())
}

/// Creates a new project in the current directory.
///
/// # Returns
/// `Ok(())` if successful, `Err(String)` with error message if failed
fn create_project_in_current_dir() -> Result<(), String> {
    let src_path = Path::new("src");
    let main_path = src_path.join("main.sm");
    let toml_path = Path::new("Summit.toml");

    if main_path.exists() || toml_path.exists() {
        return Err("A Summit project already exists in this directory (src/main.sm or Summit.toml found)".to_string());
    }

    print!("Create a new Summit project in the current directory? [y/N]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;

    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("Aborted.");
        return Ok(());
    }

    if !src_path.exists() {
        fs::create_dir(src_path)
            .map_err(|e| format!("Failed to create src directory: {}", e))?;
    }

    fs::write(&main_path, MAIN_SM_TEMPLATE)
        .map_err(|e| format!("Failed to create main.sm: {}", e))?;
    
    let current_dir_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "summit_project".to_string());

    let config = SummitConfig::default_config(&current_dir_name);
    config.save(toml_path)?;

    println!("Created new Summit project in current directory");
    println!();
    println!("To get started:");
    println!("  summit run");

    Ok(())
}