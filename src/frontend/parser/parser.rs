use crate::frontend::lexer::Token;
use crate::frontend::ast::*;
use super::statement_parser::StatementParser;
use super::declaration_parser::DeclarationParser;

/// The main parser that converts tokens into the Abstract Syntax Tree.
pub struct Parser {
    /// All the tokens to parse
    pub tokens: Vec<Token>,
    /// Current position in the token stream
    pub pos: usize,
}

impl Parser {
    /// Creates a new parser from a list of tokens.
    ///
    /// # Parameters
    /// - `tokens`: The tokens to parse
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Gets the current token at the parser's position.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    ///
    /// # Returns
    /// The current token
    pub fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    /// Looks ahead at a token without consuming it.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `offset`: How many tokens to look ahead
    ///
    /// # Returns
    /// The token at the specified offset, or `Token::Eof` if at the end
    pub fn peek(&self, offset: usize) -> &Token {
        if self.pos + offset < self.tokens.len() {
            &self.tokens[self.pos + offset]
        } else {
            &Token::Eof
        }
    }

    /// Advances to the next token.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    pub fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    /// Expects a specific token at the current position.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `expected`: The token that should be next
    ///
    /// # Returns
    /// - `Ok(())` if the expected token matches
    /// - `Err(String)` with an error message if it doesn't match
    pub fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.current() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, self.current()))
        }
    }

    /// Parses a complete Summit program.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Program` AST or an error message
    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut imports = Vec::new();
        let mut globals = Vec::new();
        let mut statements = Vec::new();
        let mut functions = Vec::new();

        while self.current() != &Token::Eof {
            match self.current() {
                Token::Import => {
                    let mut decl_parser = DeclarationParser::new(self);
                    imports.push(decl_parser.parse_import()?);
                }
                Token::Struct => {
                    let mut decl_parser = DeclarationParser::new(self);
                    globals.push(decl_parser.parse_struct()?);
                }
                Token::Var => {
                    let mut decl_parser = DeclarationParser::new(self);
                    globals.push(decl_parser.parse_global_var()?);
                }
                Token::Const => {
                    let mut decl_parser = DeclarationParser::new(self);
                    globals.push(decl_parser.parse_global_const()?);
                }
                Token::Comptime => {
                    let mut decl_parser = DeclarationParser::new(self);
                    globals.push(decl_parser.parse_global_comptime()?);
                }
                Token::Func => {
                    let mut decl_parser = DeclarationParser::new(self);
                    functions.push(decl_parser.parse_func()?);
                }
                Token::Extern => {
                    let mut decl_parser = DeclarationParser::new(self);
                    functions.push(decl_parser.parse_func()?);
                }
                _ => {
                    let mut stmt_parser = StatementParser::new(self);
                    statements.push(stmt_parser.parse_stmt()?);
                }
            }
        }

        Ok(Program { imports, globals, statements, functions })
    }
}

/// Parses tokens into a program AST.
///
/// # Parameters
/// - `tokens`: The tokens to parse
///
/// # Returns
/// A `Program` AST or an error message
pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}