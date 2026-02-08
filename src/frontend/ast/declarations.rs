use super::expressions::Expression;
use super::statements::Statement;

/// A global declaration that appears at the top level of a program.
#[derive(Debug, Clone)]
pub enum GlobalDeclaration {
    /// A global constant: `const {name}: {type} = {value};`
    Const {
        name: String,
        var_type: Option<String>,
        value: Expression
    },

    /// A global compile-time variable: `comptime {name}: {type} = {value};`
    Comptime {
        name: String,
        var_type: Option<String>,
        value: Expression
    },
}

/// An import statement that brings in other modules.
#[derive(Debug, Clone)]
pub struct Import {
    /// The module path to import: `module::submodule::function`
    pub path: Vec<String>,
}

/// A function definition.
#[derive(Debug, Clone)]
pub struct Function {
    /// The function's name
    pub name: String,

    /// The function's parameters
    pub params: Vec<Parameter>,

    /// The type the function returns
    pub return_type: String,

    /// The statements inside the function body
    pub body: Vec<Statement>,
}

/// A parameter in a function definition.
#[derive(Debug, Clone)]
pub struct Parameter {
    /// The parameter's name
    pub name: String,

    /// The parameter's type
    pub param_type: String,
}