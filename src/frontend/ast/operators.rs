/// All the different binary operators in Summit.
///
/// These are the operators that work with two values, like `a + b`
/// or `x == y`.
#[derive(Debug, Clone)]
pub enum BinaryOp {
    /// Addition: `+`
    Add,

    /// Subtraction: `-`
    Sub,

    /// Multiplication: `*`
    Mul,

    /// Division: `/`
    Div,

    /// Modulo (remainder): `%`
    Mod,

    /// Equality: `==`
    Equal,

    /// Inequality: `!=`
    NotEqual,

    /// Less than: `<`
    Less,

    /// Greater than: `>`
    Greater,

    /// Less than or equal: `<=`
    LessEqual,

    /// Greater than or equal: `>=`
    GreaterEqual,

    /// Logical AND: `and`
    And,

    /// Logical OR: `or`
    Or,
}

/// All the different unary operators in Summit.
///
/// These are the operators that work with one value, like `-x`
/// or `not y`.
#[derive(Debug, Clone)]
pub enum UnaryOp {
    /// Numeric negation: `-`
    Negate,

    /// Logical NOT: `not`
    Not,
}