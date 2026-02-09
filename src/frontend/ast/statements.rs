use super::expressions::Expression;

/// All the different kinds of statements in Summit.
///
/// Statements are the building blocks of Summit programs, they're
/// the things that actually "do stuff" in your code.
#[derive(Debug, Clone)]
pub enum Statement {
    /// A variable declaration: `var {name}: {type} = {value};`
    Var {
        name: String,
        var_type: Option<String>,
        value: Expression
    },

    /// A constant declaration: `const {name}: {type} = {value};`
    Const {
        name: String,
        var_type: Option<String>,
        value: Expression
    },

    /// A compile-time variable: `comptime {name}: {type} = {value};`
    Comptime {
        name: String,
        var_type: Option<String>,
        value: Expression
    },

    /// A variable assignment: `{name} = {value};`
    Assign {
        name: String,
        value: Expression
    },

    /// A return statement: `ret {expression};`
    Return(Expression),

    /// An expression statement: `expression;`
    Expression(Expression),

    /// An if statement: `if condition { ... } else { ... }`
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>
    },

    /// A while loop: `while condition { ... }`
    While {
        condition: Expression,
        body: Vec<Statement>
    },

    /// A for loop.
    /// Formats:
    /// - `for var {x} in {start_range} to {end_range} { ... }`
    /// - `for var {x} in {start_range} through {end_range} { ... }`
    /// - `for var {x} in {start_range} to {end_range} where {expression} { ... }`
    /// - `for var {x} in {start_range} through {end_range} where {expression} { ... }`
    /// - `for var {x} in {start_range} to {end_range} by {number} { ... }`
    /// - `for var {x} in {start_range} through {end_range} by {number} { ... }`
    For {
        variable: String,
        start: Expression,
        end: Expression,
        inclusive: bool,
        step: Option<Expression>,
        filter: Option<Expression>,
        body: Vec<Statement>
    },
}