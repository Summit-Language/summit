use std::fs;
use std::path::Path;
use std::process::Command;
use crate::compiler::Compiler;
use crate::utils::args::CompilerConfig;

/// Compiles and runs a Summit program in one step.
///
/// # Arguments
/// * `file` - The Summit source file to compile and run
///
/// # Returns
/// * `Ok(())` if the program compiles and runs successfully
/// * `Err(String)` with an error message if something goes wrong
pub fn run_project(file: &str) -> Result<(), String> {
    let input_path = Path::new(file);

    if !input_path.exists() {
        return Err(format!("File not found: {}", file));
    }

    let temp_dir = std::env::temp_dir().join("summit_run");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let file_stem = input_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let c_output = temp_dir.join(format!("{}.c", file_stem));
    let exe_output = temp_dir.join(file_stem);

    let config = CompilerConfig {
        input_file: input_path.to_path_buf(),
        c_output_file: c_output.clone(),
        output_file: Some(exe_output.clone()),
    };

    let mut compiler = Compiler::new(config);
    compiler.compile()?;

    Command::new(&exe_output)
        .status()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    let _ = fs::remove_file(&c_output);
    let _ = fs::remove_file(&exe_output);

    Ok(())
}