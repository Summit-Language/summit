use crate::compiler::Compiler;
use crate::utils::args::{parse_args, CompilerConfig};
use crate::config::SummitConfig;
use std::path::Path;
use std::fs;

/// Builds a project from command line arguments or Summit.toml.
///
/// # Parameters
/// - `args`: Optional command-line arguments. If empty and Summit.toml exists, uses config.
///
/// # Returns
/// Ok(()) if build succeeds, Err with message on failure
pub fn build_project(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        let toml_path = Path::new("Summit.toml");
        if toml_path.exists() {
            return build_from_config(toml_path);
        } else {
            return Err("No input file specified and no Summit.toml found. Usage: summit build <input.sm>".to_string());
        }
    }

    let mut full_args = vec!["summit".to_string()];
    full_args.extend_from_slice(args);

    let config = parse_args(&full_args)?;

    let mut compiler = Compiler::new(config);
    compiler.compile()?;

    Ok(())
}

/// Builds from a Summit.toml configuration file.
fn build_from_config(toml_path: &Path) -> Result<(), String> {
    let config = SummitConfig::load(toml_path)?;

    let input_file = config.project.main.clone();
    let input_path = Path::new(&input_file);

    if !input_path.exists() {
        return Err(format!("Main file not found: {}", input_file));
    }

    let build_dir = Path::new(&config.build.output_dir);
    let objects_dir = build_dir.join("objects");
    let intermediate_dir = build_dir.join("intermediate");

    fs::create_dir_all(&objects_dir)
        .map_err(|e| format!("Failed to create {}/objects directory: {}",
                             config.build.output_dir, e))?;
    fs::create_dir_all(&intermediate_dir)
        .map_err(|e| format!("Failed to create {}/intermediate directory: {}",
                             config.build.output_dir, e))?;

    let output_name = config.get_output_name();
    let output_path = build_dir.join(&output_name);

    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    let c_output = intermediate_dir.join(format!("{}.c", file_stem));

    let compiler_config = CompilerConfig {
        input_file: input_path.to_path_buf(),
        c_output_file: c_output.clone(),
        output_file: Some(output_path.clone()),
    };

    let mut compiler = Compiler::new(compiler_config);
    compiler.compile()?;

    println!("Built project: {}", config.project.name);
    println!("  Output: {}", output_path.display());

    Ok(())
}