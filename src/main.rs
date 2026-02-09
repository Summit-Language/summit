mod frontend;
mod analysis;
mod compiler;
mod utils;
mod commands;
mod config;

use std::env;
use std::process::exit;

use commands::{new_project, run_project, build_project, clean_project};

/// The main entry point for the Summit compiler.
///
/// Handles command line arguments and decides what to do
/// based on the user's input arguments.
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "new" => {
            let project_name = if args.len() > 2 {
                Some(args[2].clone())
            } else {
                None
            };

            if let Err(e) = new_project(project_name) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        "run" => {
            let file = if args.len() > 2 {
                Some(args[2].as_str())
            } else {
                None
            };

            if let Err(e) = run_project(file) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        "build" => {
            let build_args = if args.len() > 2 {
                &args[2..]
            } else {
                &[]
            };

            if let Err(e) = build_project(build_args) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        "clean" => {
            if let Err(e) = clean_project() {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            exit(1);
        }
    }
}

/// Shows how to use the Summit compiler.
fn print_usage() {
    eprintln!("Summit Language Compiler");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  summit new [project_name]    Create a new Summit project");
    eprintln!("  summit run [file]            Compile and run a Summit file (default: uses Summit.toml)");
    eprintln!("  summit build [file]          Compile a Summit file (default: uses Summit.toml)");
    eprintln!("  summit clean                 Remove the built binaries");
}