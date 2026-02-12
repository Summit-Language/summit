use super::expressions::Expression;

/// All the different kinds of statements in Summit.
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

    /// A field assignment: `point.x = 5;`
    FieldAssign {
        object: String,
        field: String,
        value: Expression,
    },

    /// A return statement: `ret {expression};`
    Return(Expression),

    /// An expression statement: `expression;`
    Expression(Expression),

    /// An if statement: `if condition { ... } elseif condition { ... } else { ... }`
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        elseif_blocks: Vec<ElseIfBlock>,
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

    /// A when statement (switch/match): `when value { is 1: { ... } is 2: { ... } else { ... } }`
    When {
        value: Expression,
        cases: Vec<WhenCase>,
        else_block: Option<Vec<Statement>>
    },

    /// An expect statement: `expect condition { else -> ... }` or `expect value is pattern { else -> ... }`
    Expect {
        condition: Expression,
        pattern: Option<ExpectPattern>,
        else_block: Vec<Statement>
    },

    /// A next statement (continue): `next;`
    Next,

    /// A stop statement (break): `stop;`
    Stop,

    /// A fallthrough statement: `fallthrough;`
    Fallthrough,
}

/// Represents an elseif block in an if statement
#[derive(Debug, Clone)]
pub struct ElseIfBlock {
    /// The condition to test
    pub condition: Expression,
    /// The statements to execute if the condition is true
    pub body: Vec<Statement>,
}

/// Represents a single case in a when statement
#[derive(Debug, Clone)]
pub struct WhenCase {
    /// The pattern to match, can be a single value or a range
    pub pattern: WhenPattern,
    /// The statements to execute if the pattern matches
    pub body: Vec<Statement>,
}

/// Represents a pattern in a when case
#[derive(Debug, Clone)]
pub enum WhenPattern {
    /// Single value pattern
    Single(Expression),
    /// Range pattern
    Range {
        start: Expression,
        end: Expression,
        inclusive: bool,
    },
}

/// Represents a pattern in an expect statement
#[derive(Debug, Clone)]
pub enum ExpectPattern {
    /// Single value pattern
    Single(Expression),
    /// Range pattern
    Range {
        start: Expression,
        end: Expression,
        inclusive: bool,
    },
}