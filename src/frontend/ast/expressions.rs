use crate::frontend::ast::WhenPattern;
use super::operators::{BinaryOp, UnaryOp};
use super::structs::StructFieldInit;

/// All the different kinds of expressions in Summit.
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
    IfExpr {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>
    },

    /// A when expression: `when value { is pattern -> expr, ... else -> expr }`
    ///
    /// Returns a value based on pattern matching.
    WhenExpr {
        value: Box<Expression>,
        cases: Vec<WhenExprCase>,
        else_expr: Box<Expression>
    },

    /// A struct instantiation: `Vector2 { x: 3, y: 5 }` or `Vector2 { 3, 5 }`
    StructInit {
        struct_name: String,
        fields: Vec<StructFieldInit>,
    },

    /// A field access: `point.x`
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },

    EnumConstruct {
        enum_name: String,
        variant_name: String,
        args: Vec<Expression>,
    },
}

/// A single case in a when expression
#[derive(Debug, Clone)]
pub struct WhenExprCase {
    pub pattern: WhenPattern,
    pub result: Expression,
}