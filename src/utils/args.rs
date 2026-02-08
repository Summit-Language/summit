use std::path::PathBuf;

/// Settings for the compiler, tells it what to compile and where to put things.
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// The Summit source code file to compile
    pub input_file: PathBuf,

    /// Where to put the final executable
    pub output_file: Option<PathBuf>,

    /// Where to put the generated C code
    pub c_output_file: PathBuf,
}

/// Reads command line arguments and figures out what the user wants to compile.
///
/// # Arguments
/// * `args` - The command line arguments
///
/// # Returns
/// * `Ok(CompilerConfig)` if the arguments make sense
/// * `Err(String)` with an error message if something's wrong
pub fn parse_args(args: &[String]) -> Result<CompilerConfig, String> {
    if args.len() < 2 {
        return Err("No input file provided".to_string());
    }

    let input_file = PathBuf::from(&args[1]);

    if !input_file.exists() {
        return Err(format!("Input file '{}' does not exist", input_file.display()));
    }

    let c_output_file = input_file.with_extension("c");
    let output_file = Some(input_file.with_extension(""));

    Ok(CompilerConfig {
        input_file,
        output_file,
        c_output_file,
    })
}