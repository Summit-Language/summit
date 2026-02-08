use super::declarations::{Import, GlobalDeclaration, Function};
use super::statements::Statement;

/// A complete Summit program.
///
/// This is the root of the Abstract Syntax Tree, it contains
/// everything that makes up a Summit program.
#[derive(Debug, Clone)]
pub struct Program {
    /// All the imports at the top of the program
    pub imports: Vec<Import>,

    /// All the global declarations (constants and comptime variables)
    pub globals: Vec<GlobalDeclaration>,

    /// All the top level statements that run when the program starts
    pub statements: Vec<Statement>,

    /// All the functions defined in the program
    pub functions: Vec<Function>,
}