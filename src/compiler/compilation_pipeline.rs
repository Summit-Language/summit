use std::fs;
use std::process::Command;
use crate::utils::args::CompilerConfig;
use crate::utils::path::get_stdlib_path;
use crate::frontend;
use crate::analysis;
use crate::compiler::backend;

/// The Summit compiler. This is what actually compiles the Summit code.
///
/// It holds the compilation settings and runs the whole process from
/// start to finish.
pub struct Compiler {
    /// Your compilation settings, things like which file to compile
    /// and where to put the output.
    config: CompilerConfig,
}

impl Compiler {
    /// Creates a new compiler instance with the given configuration.
    ///
    /// # Parameters
    /// - `config`: Configuration for this compilation session
    ///
    /// # Returns
    /// A new `Compiler` instance
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Compiles a Summit source file into an executable.
    ///
    /// This runs the complete compilation pipeline which leads to your
    /// executable file.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// - `Ok(())` if compilation succeeds
    /// - `Err(String)` with error message if compilation fails
    pub fn compile(&mut self) -> Result<(), String> {
        let source = fs::read_to_string(&self.config.input_file)
            .map_err(|e| format!("Error reading file: {}", e))?;

        let tokens = frontend::tokenize(&source)
            .map_err(|e| format!("Lexer error: {}", e))?;

        let ast = frontend::parse(tokens)
            .map_err(|e| format!("Parser error: {}", e))?;

        analysis::analyze(&ast)
            .map_err(|e| format!("Semantic error: {}", e))?;

        let c_code = backend::generate(&ast, self.config.link_libs.clone());

        fs::write(&self.config.c_output_file, c_code)
            .map_err(|e| format!("Error writing C file: {}", e))?;

        self.compile_c()?;

        Ok(())
    }

    /// Compiles the generated C code to an executable using GCC.
    ///
    /// Links the Summit standard library and sets appropriate
    /// library paths for runtime linking.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    ///
    /// # Returns
    /// - `Ok(())` if C compilation succeeds
    /// - `Err(String)` with error message if GCC fails
    fn compile_c(&self) -> Result<(), String> {
        let output_file = self.config.output_file.as_ref()
            .ok_or("No output file specified")?;

        let linking_summitstd = self.config.link_libs.contains(&"summitstd".to_string());
        let linking_libc = self.config.link_libs.contains(&"c".to_string());

        let stdlib_path = get_stdlib_path();

        if linking_summitstd {
            let lib_path = std::path::Path::new(&stdlib_path).join("libsummit_std.so");
            if !lib_path.exists() {
                return Err(format!(
                    "Standard library not found at: {}\n\
             Library file missing: {}\n\
             Please set SUMMIT_STDLIB_PATH environment variable correctly.",
                    stdlib_path, lib_path.display()
                ));
            }
        }

        let include_path = std::path::Path::new(&stdlib_path).join("include");

        let header_path = include_path.join("freestanding.h");
        if !header_path.exists() {
            return Err(format!(
                "Header file not found at: {}\n\
         Please reinstall the standard library with 'make install'.",
                header_path.display()
            ));
        }

        let mut gcc_args = vec![
            self.config.c_output_file.to_str().unwrap(),
            "-I", include_path.to_str().unwrap(),
            "-std=c11",
            "-fno-builtin",
            "-o", output_file.to_str().unwrap(),
        ];

        if linking_summitstd {
            gcc_args.extend(["-L", &stdlib_path]);
        }

        if !linking_libc {
            gcc_args.extend([
                "-ffreestanding",
                "-fno-builtin",
                "-nostdlib",
                "-nostartfiles",
                "-nodefaultlibs",
                "-fno-stack-protector",
                "-mno-red-zone",
            ]);
        }

        for lib in &self.config.link_libs {
            if lib == "summitstd" {
                gcc_args.push("-lsummit_std");
            } else {
                gcc_args.push("-l");
                gcc_args.push(lib);
            }
        }

        if linking_summitstd {
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            gcc_args.extend(["-Wl,-rpath", &stdlib_path]);
            gcc_args.extend(["-Wl,-L", &stdlib_path]);
        }

        let status = Command::new("gcc")
            .args(&gcc_args)
            .status()
            .map_err(|e| format!("Error compiling C code: {}", e))?;

        if !status.success() {
            return Err("Compilation failed".to_string());
        }

        println!("Compiled successfully: {}", output_file.display());
        Ok(())
    }
}