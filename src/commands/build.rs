use crate::compiler::Compiler;
use crate::utils::args::{parse_args};

/// Builds a project from command line arguments.
///
/// # Parameters
/// - `args`: Command-line arguments
///
/// # Returns
/// Ok(()) if build succeeds, Err with message on failure
pub fn build_project(args: &[String]) -> Result<(), String> {
    let mut full_args = vec!["summit".to_string()];
    full_args.extend_from_slice(args);

    let config = parse_args(&full_args)?;

    let mut compiler = Compiler::new(config);
    compiler.compile()?;

    Ok(())
}