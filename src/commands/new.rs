use std::fs;
use std::io::{self, Write};
use std::path::Path;

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

    if main_path.exists() {
        return Err("A Summit project already exists in this directory (src/main.sm found)".to_string());
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

    println!("Created new Summit project in current directory");
    println!();
    println!("To get started:");
    println!("  summit run");

    Ok(())
}