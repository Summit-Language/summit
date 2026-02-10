use std::fmt;

/// All possible tokens in the Summit language.
///
/// This enum represents every token that the lexer can produce.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Import,
    Func,
    Ret,
    Var,
    Const,
    Comptime,
    If,
    Else,
    While,
    For,
    Next,
    Stop,
    When,
    Expect,
    Is,
    Fallthrough,
    In,
    To,
    Through,
    By,
    Where,
    Not,
    And,
    Or,
    Null,
    True,
    False,

    // Types
    Type(String),

    // Identifiers and literals
    Identifier(String),
    IntLiteral(u128),
    StringLiteral(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    Equal,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Question,
    Arrow,

    // Delimiters
    DoubleColon,
    Colon,
    Semicolon,
    Comma,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftAngle,
    RightAngle,

    // Special
    Eof,
}

impl fmt::Display for Token {
    /// Converts a token to a string for display purposes.
    ///
    /// # Parameters
    /// - `self`: The token to display
    /// - `f`: The formatter to write to
    ///
    /// # Returns
    /// Formatting result
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}