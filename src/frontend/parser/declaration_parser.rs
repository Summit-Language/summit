use crate::frontend::lexer::Token;
use crate::frontend::ast::*;
use super::parser::Parser;
use super::expression_parser::ExpressionParser;
use super::statement_parser::StatementParser;

/// Parses declarations at the global/root level of a Summit program.
pub struct DeclarationParser<'a> {
    parser: &'a mut Parser,
}

impl<'a> DeclarationParser<'a> {
    /// Creates a new DeclarationParser for the given Parser.
    pub fn new(parser: &'a mut Parser) -> Self {
        DeclarationParser { parser }
    }

    /// Parses a global variable declaration.
    ///
    /// Format: `var {name}: {type} = {value};`
    ///
    /// # Parameters:
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `GlobalDeclaration::Var` if successful, or an error message.
    pub fn parse_global_var(&mut self) -> Result<GlobalDeclaration, String> {
        self.parser.expect(Token::Var)?;

        let name = if let Token::Identifier(n) = self.parser.current() {
            let name = n.clone();
            self.parser.advance();
            name
        } else {
            return Err("Expected variable name".to_string());
        };

        let var_type = if self.parser.current() == &Token::Colon {
            self.parser.advance();
            if let Token::Type(t) = self.parser.current() {
                let typ = t.clone();
                self.parser.advance();
                Some(typ)
            } else {
                return Err("Expected type after colon".to_string());
            }
        } else {
            None
        };

        self.parser.expect(Token::Assign)?;
        let mut expr_parser = ExpressionParser::new(self.parser);
        let value = expr_parser.parse_expr()?;
        self.parser.expect(Token::Semicolon)?;

        Ok(GlobalDeclaration::Var { name, var_type, value })
    }

    /// Parses a global constant declaration.
    ///
    /// Format: `const {name}: {type} = {value};`
    ///
    /// # Parameters:
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `GlobalDeclaration::Const` if successful, or an error message.
    pub fn parse_global_const(&mut self) -> Result<GlobalDeclaration, String> {
        self.parser.expect(Token::Const)?;

        let name = if let Token::Identifier(n) = self.parser.current() {
            let name = n.clone();
            self.parser.advance();
            name
        } else {
            return Err("Expected variable name".to_string());
        };

        let var_type = if self.parser.current() == &Token::Colon {
            self.parser.advance();
            if let Token::Type(t) = self.parser.current() {
                let typ = t.clone();
                self.parser.advance();
                Some(typ)
            } else {
                return Err("Expected type after colon".to_string());
            }
        } else {
            None
        };

        self.parser.expect(Token::Assign)?;
        let mut expr_parser = ExpressionParser::new(self.parser);
        let value = expr_parser.parse_expr()?;
        self.parser.expect(Token::Semicolon)?;

        Ok(GlobalDeclaration::Const { name, var_type, value })
    }

    /// Parses a compile-time variable declaration.
    ///
    /// Format: `comptime {name}: {type} = {value};`
    ///
    /// # Parameters:
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `GlobalDeclaration::Comptime` if successful, or an error message.
    pub fn parse_global_comptime(&mut self) -> Result<GlobalDeclaration, String> {
        self.parser.expect(Token::Comptime)?;

        let name = if let Token::Identifier(n) = self.parser.current() {
            let name = n.clone();
            self.parser.advance();
            name
        } else {
            return Err("Expected variable name".to_string());
        };

        let var_type = if self.parser.current() == &Token::Colon {
            self.parser.advance();
            if let Token::Type(t) = self.parser.current() {
                let typ = t.clone();
                self.parser.advance();
                Some(typ)
            } else {
                return Err("Expected type after colon".to_string());
            }
        } else {
            None
        };

        self.parser.expect(Token::Assign)?;
        let mut expr_parser = ExpressionParser::new(self.parser);
        let value = expr_parser.parse_expr()?;
        self.parser.expect(Token::Semicolon)?;

        Ok(GlobalDeclaration::Comptime { name, var_type, value })
    }

    /// Parses an import statement.
    ///
    /// Format: `import module::submodule;`
    ///
    /// # Parameters:
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Import` struct with the module path, or an error message.
    pub fn parse_import(&mut self) -> Result<Import, String> {
        self.parser.expect(Token::Import)?;

        let mut path = Vec::new();

        if let Token::Identifier(name) = self.parser.current() {
            path.push(name.clone());
            self.parser.advance();
        } else {
            return Err("Expected module name".to_string());
        }

        while self.parser.current() == &Token::DoubleColon {
            self.parser.advance();
            if let Token::Identifier(name) = self.parser.current() {
                path.push(name.clone());
                self.parser.advance();
            } else {
                return Err("Expected module name after ::".to_string());
            }
        }

        self.parser.expect(Token::Semicolon)?;
        Ok(Import { path })
    }

    /// Parses a function declaration.
    ///
    /// Format: `func name(param: type): return_type { ... }`
    ///
    /// # Parameters:
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Function` struct with the function details, or an error message.
    pub fn parse_func(&mut self) -> Result<Function, String> {
        self.parser.expect(Token::Func)?;

        let name = if let Token::Identifier(n) = self.parser.current() {
            let name = n.clone();
            self.parser.advance();
            name
        } else {
            return Err("Expected function name".to_string());
        };

        self.parser.expect(Token::LeftParen)?;

        let mut params = Vec::new();
        if self.parser.current() != &Token::RightParen {
            loop {
                let param_name = if let Token::Identifier(n)
                    = self.parser.current() {
                    let name = n.clone();
                    self.parser.advance();
                    name
                } else {
                    return Err("Expected parameter name".to_string());
                };

                self.parser.expect(Token::Colon)?;

                let param_type = if let Token::Type(t) = self.parser.current() {
                    let typ = t.clone();
                    self.parser.advance();
                    typ
                } else {
                    return Err("Expected parameter type".to_string());
                };

                params.push(Parameter { name: param_name, param_type });

                if self.parser.current() == &Token::Comma {
                    self.parser.advance();
                } else {
                    break;
                }
            }
        }

        self.parser.expect(Token::RightParen)?;
        self.parser.expect(Token::Colon)?;

        let return_type = if let Token::Type(t) = self.parser.current() {
            let typ = t.clone();
            self.parser.advance();
            typ
        } else {
            return Err("Expected return type".to_string());
        };

        self.parser.expect(Token::LeftBrace)?;

        let mut body = Vec::new();
        while self.parser.current() != &Token::RightBrace {
            let mut stmt_parser = StatementParser::new(self.parser);
            body.push(stmt_parser.parse_stmt()?);
        }

        self.parser.expect(Token::RightBrace)?;

        Ok(Function { name, params, return_type, body })
    }
}