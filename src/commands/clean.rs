use crate::config::SummitConfig;
use std::path::Path;
use std::fs;

/// Cleans the build directory, removing all build artifacts.
///
/// # Returns
/// Ok(()) if clean succeeds, Err with message on failure
pub fn clean_project() -> Result<(), String> {
    let toml_path = Path::new("Summit.toml");

    let build_dir = if toml_path.exists() {
        let config = SummitConfig::load(toml_path)?;
        config.build.output_dir
    } else {
        "build".to_string()
    };

    let build_path = Path::new(&build_dir);

    if !build_path.exists() {
        println!("Nothing to clean - '{}' directory does not exist", build_dir);
        return Ok(());
    }

    fs::remove_dir_all(build_path)
        .map_err(|e| format!("Failed to remove '{}' directory: {}", build_dir, e))?;

    println!("Cleaned build artifacts from '{}'", build_dir);

    Ok(())
}