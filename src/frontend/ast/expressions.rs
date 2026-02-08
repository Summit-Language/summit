use super::operators::{BinaryOp, UnaryOp};

/// All the different kinds of expressions in Summit.
///
/// Expressions are parts of code that produce a value when they run,
/// like `x + 5` or `some_function()`.
#[derive(Debug, Clone)]
pub enum Expression {
    /// An integer literal: `42`, `100`, etc.
    IntLiteral(u128),

    /// A string literal: `"hello"`, `"world"`, etc.
    StringLiteral(String),

    /// A boolean literal: `true` or `false`
    BoolLiteral(bool),

    /// The null value: `null`
    NullLiteral,

    /// A variable reference: `x`, `my_var`, etc.
    Variable(String),

    /// A function call: `function_name(arg1, arg2)`
    ///
    /// - `path`: The module path to the function
    /// - `type_args`: Generic type arguments
    /// - `args`: The actual arguments passed to the function
    Call {
        path: Vec<String>,
        type_args: Option<Vec<String>>,
        args: Vec<Expression>
    },

    /// A binary operation: `left op right`
    ///
    /// Examples: `a + b`, `x == y`, `p and q`
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>
    },

    /// A unary operation: `op operand`
    ///
    /// Examples: `-x`, `not y`
    Unary {
        op: UnaryOp,
        operand: Box<Expression>
    },

    /// An if expression: `if condition { ... } else { ... }`
    ///
    /// Unlike an if statement, this returns a value from either
    /// the then block or the else block.
    IfExpr {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>
    },
}