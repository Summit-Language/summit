use crate::frontend::lexer::Token;
use crate::frontend::ast::*;
use super::parser::Parser;
use super::expression_parser::ExpressionParser;

/// Parses statements in Summit code.
pub struct StatementParser<'a> {
    parser: &'a mut Parser,
}

impl<'a> StatementParser<'a> {
    /// Creates a new StatementParser for the given Parser.
    ///
    /// # Parameters
    /// - `parser`: The parser to use for token handling
    pub fn new(parser: &'a mut Parser) -> Self {
        StatementParser { parser }
    }

    /// Parses a statement from the current token position.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement` or an error message.
    pub fn parse_stmt(&mut self) -> Result<Statement, String> {
        match self.parser.current() {
            Token::Var => self.parse_let(),
            Token::Const => self.parse_const(),
            Token::Comptime => self.parse_comptime(),
            Token::Ret => self.parse_return(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Identifier(_) => {
                if self.parser.peek(1) == &Token::Assign {
                    self.parse_assign()
                } else {
                    let mut expr_parser = ExpressionParser::new(self.parser);
                    let expr = expr_parser.parse_expr()?;
                    self.parser.expect(Token::Semicolon)?;
                    Ok(Statement::Expression(expr))
                }
            }
            _ => {
                let mut expr_parser = ExpressionParser::new(self.parser);
                let expr = expr_parser.parse_expr()?;
                self.parser.expect(Token::Semicolon)?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    /// Parses a for loop statement.
    /// 
    /// Formats:
    /// - `for var {x} in {start_range} to {end_range} { ... }`
    /// - `for var {x} in {start_range} through {end_range} { ... }`
    /// - `for var {x} in {start_range} to {end_range} where {expression} { ... }`
    /// - `for var {x} in {start_range} through {end_range} where {expression} { ... }`
    /// - `for var {x} in {start_range} to {end_range} by {number} { ... }`
    /// - `for var {x} in {start_range} through {end_range} by {number} { ... }`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::For` or an error message.
    fn parse_for(&mut self) -> Result<Statement, String> {
        self.parser.expect(Token::For)?;
        self.parser.expect(Token::Var)?;

        let variable = if let Token::Identifier(n) = self.parser.current() {
            let name = n.clone();
            self.parser.advance();
            name
        } else {
            return Err("Expected loop variable name".to_string());
        };

        self.parser.expect(Token::In)?;

        let mut expr_parser = ExpressionParser::new(self.parser);
        let start = expr_parser.parse_or()?;

        let inclusive = match self.parser.current() {
            Token::To => {
                self.parser.advance();
                false
            }
            Token::Through => {
                self.parser.advance();
                true
            }
            _ => return Err("Expected 'to' or 'through' after range start".to_string()),
        };

        let mut expr_parser = ExpressionParser::new(self.parser);
        let end = expr_parser.parse_or()?;

        let step = if self.parser.current() == &Token::By {
            self.parser.advance();
            let mut expr_parser = ExpressionParser::new(self.parser);
            Some(expr_parser.parse_or()?)
        } else {
            None
        };

        let filter = if self.parser.current() == &Token::Where {
            self.parser.advance();
            let mut expr_parser = ExpressionParser::new(self.parser);
            Some(expr_parser.parse_expr()?)
        } else {
            None
        };

        self.parser.expect(Token::LeftBrace)?;

        let mut body = Vec::new();
        while self.parser.current() != &Token::RightBrace {
            body.push(self.parse_stmt()?);
        }

        self.parser.expect(Token::RightBrace)?;

        Ok(Statement::For {
            variable,
            start,
            end,
            inclusive,
            step,
            filter,
            body,
        })
    }

    /// Parses a const statement.
    /// 
    /// Format: `const {name}: {type} = {value};`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::Const` or an error message.
    fn parse_const(&mut self) -> Result<Statement, String> {
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

        Ok(Statement::Const { name, var_type, value })
    }

    /// Parses a comptime statement.
    /// 
    /// Format: `comptime {name}: {type} = {value};`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::Comptime` or an error message.
    fn parse_comptime(&mut self) -> Result<Statement, String> {
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

        Ok(Statement::Comptime { name, var_type, value })
    }

    /// Parses a let variable declaration statement.
    /// 
    /// Format: `let {name}: {type} = {value};`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::Let` or an error message.
    fn parse_let(&mut self) -> Result<Statement, String> {
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

        Ok(Statement::Let { name, var_type, value })
    }

    /// Parses an assignment statement.
    /// 
    /// Format: `{variable_name} = {value};`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::Assign` or an error message.
    fn parse_assign(&mut self) -> Result<Statement, String> {
        let name = if let Token::Identifier(n) = self.parser.current() {
            let name = n.clone();
            self.parser.advance();
            name
        } else {
            return Err("Expected variable name".to_string());
        };

        self.parser.expect(Token::Assign)?;
        let mut expr_parser = ExpressionParser::new(self.parser);
        let value = expr_parser.parse_expr()?;
        self.parser.expect(Token::Semicolon)?;

        Ok(Statement::Assign { name, value })
    }

    /// Parses a return statement.
    /// 
    /// Format: `ret {expr};`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::Return` or an error message.
    fn parse_return(&mut self) -> Result<Statement, String> {
        self.parser.expect(Token::Ret)?;
        let mut expr_parser = ExpressionParser::new(self.parser);
        let expr = expr_parser.parse_expr()?;
        self.parser.expect(Token::Semicolon)?;
        Ok(Statement::Return(expr))
    }

    /// Parses an if statement.
    /// 
    /// Format: `if condition { ... } elseif condition { ... } else { ... }`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::If` or an error message.
    fn parse_if(&mut self) -> Result<Statement, String> {
        self.parser.expect(Token::If)?;
        let mut expr_parser = ExpressionParser::new(self.parser);
        let condition = expr_parser.parse_expr()?;
        self.parser.expect(Token::LeftBrace)?;

        let mut then_block = Vec::new();
        while self.parser.current() != &Token::RightBrace {
            then_block.push(self.parse_stmt()?);
        }
        self.parser.expect(Token::RightBrace)?;

        let else_block = if self.parser.current() == &Token::Else {
            self.parser.advance();
            self.parser.expect(Token::LeftBrace)?;
            let mut else_stmts = Vec::new();
            while self.parser.current() != &Token::RightBrace {
                else_stmts.push(self.parse_stmt()?);
            }
            self.parser.expect(Token::RightBrace)?;
            Some(else_stmts)
        } else {
            None
        };

        Ok(Statement::If { condition, then_block, else_block })
    }

    /// Parses a while statement.
    /// 
    /// Format: `while {condition} { ... }`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::While` or an error message.
    fn parse_while(&mut self) -> Result<Statement, String> {
        self.parser.expect(Token::While)?;
        let mut expr_parser = ExpressionParser::new(self.parser);
        let condition = expr_parser.parse_expr()?;
        self.parser.expect(Token::LeftBrace)?;

        let mut body = Vec::new();
        while self.parser.current() != &Token::RightBrace {
            body.push(self.parse_stmt()?);
        }
        self.parser.expect(Token::RightBrace)?;

        Ok(Statement::While { condition, body })
    }
}