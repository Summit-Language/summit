use crate::frontend::ast::*;
use super::program_generator::ProgramGenerator;

/// Generates C code from a Summit program AST.
///
/// # Parameters
/// - `program`: The complete program AST to generate code for
/// - `link_libs`: Libraries to link against (e.g., vec!["c".to_string()] for libc)
///
/// # Returns
/// The generated C code as a string
pub fn generate(program: &Program, link_libs: Vec<String>) -> String {
    let mut generator = ProgramGenerator::new(link_libs);
    generator.generate_program(program)
}