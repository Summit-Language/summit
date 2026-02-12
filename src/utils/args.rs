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

    /// Additional libraries to link with -l flag
    pub link_libs: Vec<String>,
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

    let mut input_file = None;
    let mut link_libs = Vec::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "build" | "run" => {
                i += 1;
            }
            "-l" => {
                if i + 1 < args.len() {
                    link_libs.push(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err(format!("Missing library name after {}", args[i]));
                }
            }
            arg if arg.starts_with("-l") => {
                let lib = &arg[2..];
                if !lib.is_empty() {
                    link_libs.push(lib.to_string());
                } else {
                    return Err("Empty library name after -l".to_string());
                }
                i += 1;
            }
            _ => {
                if input_file.is_none() {
                    input_file = Some(PathBuf::from(&args[i]));
                }
                i += 1;
            }
        }
    }

    let input_file = input_file.ok_or("No input file provided".to_string())?;

    if !input_file.exists() {
        return Err(format!("Input file '{}' does not exist", input_file.display()));
    }

    let c_output_file = input_file.with_extension("c");
    let output_file = Some(input_file.with_extension(""));

    Ok(CompilerConfig {
        input_file,
        output_file,
        c_output_file,
        link_libs,
    })
}

/// Extracts library flags from command line arguments.
/// Returns the libraries and the remaining args without the -l flags.
///
/// # Arguments
/// * `args` - The command line arguments
///
/// # Returns
/// A tuple of (link_libs, remaining_args)
pub fn extract_link_libs(args: &[String]) -> (Vec<String>, Vec<String>) {
    let mut link_libs = Vec::new();
    let mut remaining_args = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-l" => {
                if i + 1 < args.len() {
                    link_libs.push(args[i + 1].clone());
                    i += 2;
                } else {
                    // Keep the malformed -l flag in remaining args
                    remaining_args.push(args[i].clone());
                    i += 1;
                }
            }
            arg if arg.starts_with("-l") => {
                let lib = &arg[2..];
                if !lib.is_empty() {
                    link_libs.push(lib.to_string());
                }
                i += 1;
            }
            _ => {
                remaining_args.push(args[i].clone());
                i += 1;
            }
        }
    }

    (link_libs, remaining_args)
}
