use std::path::Path;
use std::process::Command;
use crate::config::SummitConfig;
use crate::commands::build_project;

/// Compiles and runs a Summit program in one step.
///
/// # Arguments
/// * `file` - Optional Summit source file to compile and run. If None, uses Summit.toml config.
///
/// # Returns
/// * `Ok(())` if the program compiles and runs successfully
/// * `Err(String)` with an error message if something goes wrong
pub fn run_project(file: Option<&str>) -> Result<(), String> {
    println!("Building project...");

    if let Some(f) = file {
        build_project(&[f.to_string()])?;
    } else {
        let toml_path = Path::new("Summit.toml");
        if !toml_path.exists() {
            return Err("No Summit.toml found in current directory. Run 'summit new' to create a project or specify a file.".to_string());
        }
        build_project(&[])?;
    }

    println!();

    let executable_path = if file.is_some() {
        let filename = Path::new(file.unwrap())
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        Path::new(&format!("./{}", filename)).to_path_buf()
    } else {
        let toml_path = Path::new("Summit.toml");
        let config = SummitConfig::load(toml_path)?;
        let build_dir = Path::new(&config.build.output_dir);

        let output_name = config.get_output_name();
        build_dir.join(&output_name)
    };

    println!("Running {}...", executable_path.display());
    println!();

    let status = Command::new(&executable_path)
        .status()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !status.success() {
        return Err(format!("Program exited with code: {}", status.code().unwrap_or(-1)));
    }

    Ok(())
}