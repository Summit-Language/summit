use crate::frontend::ast::*;
use super::program_generator::ProgramGenerator;

/// Generates C code from a Summit program AST.
///
/// # Parameters
/// - `program`: The complete program AST to generate code for
///
/// # Returns
/// The generated C code as a string
pub fn generate(program: &Program) -> String {
    let mut generator = ProgramGenerator::new();
    generator.generate_program(program)
}