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

    /// Parses a type, which can be either a built-in type or a struct/enum identifier.
    ///
    /// # Returns
    /// The type name as a String, or an error message.
    fn parse_type(&mut self) -> Result<String, String> {
        match self.parser.current() {
            Token::Type(t) => {
                let typ = t.clone();
                self.parser.advance();
                Ok(typ)
            }
            Token::Identifier(id) => {
                let typ = id.clone();
                self.parser.advance();
                Ok(typ)
            }
            _ => Err("Expected type or struct/enum name".to_string())
        }
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
            Token::Var => self.parse_var(),
            Token::Const => self.parse_const(),
            Token::Comptime => self.parse_comptime(),
            Token::Ret => self.parse_return(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::When => self.parse_when(),
            Token::Expect => self.parse_expect(),
            Token::For => self.parse_for(),
            Token::Next => self.parse_next(),
            Token::Stop => self.parse_stop(),
            Token::Fallthrough => self.parse_fallthrough(),
            Token::Identifier(_) => {
                if self.parser.peek(1) == &Token::Dot {
                    self.parse_field_assign()
                } else if self.parser.peek(1) == &Token::Assign {
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

    /// Parses a field assignment statement with support for chained field access.
    ///
    /// Format: `object.field = value;` or `object.field.subfield = value;`
    ///
    /// # Returns
    /// A `Statement::FieldAssign` or an error message.
    fn parse_field_assign(&mut self) -> Result<Statement, String> {
        // Parse the base object
        let object = if let Token::Identifier(n) = self.parser.current() {
            let name = n.clone();
            self.parser.advance();
            name
        } else {
            return Err("Expected object name".to_string());
        };

        let mut fields = Vec::new();

        while self.parser.current() == &Token::Dot {
            self.parser.advance();

            if let Token::Identifier(n) = self.parser.current() {
                fields.push(n.clone());
                self.parser.advance();
            } else {
                return Err("Expected field name after '.'".to_string());
            }
        }

        if fields.is_empty() {
            return Err("Expected at least one field access".to_string());
        }

        self.parser.expect(Token::Assign)?;

        let mut expr_parser = ExpressionParser::new(self.parser);
        let value = expr_parser.parse_expr()?;

        self.parser.expect(Token::Semicolon)?;

        if fields.len() == 1 {
            Ok(Statement::FieldAssign {
                object,
                field: fields[0].clone(),
                value
            })
        } else {
            let mut field_expr = Expression::Variable(object.clone());

            for i in 0..fields.len() - 1 {
                field_expr = Expression::FieldAccess {
                    object: Box::new(field_expr),
                    field: fields[i].clone(),
                };
            }

            let full_field = fields.join(".");

            Ok(Statement::FieldAssign {
                object,
                field: full_field,
                value
            })
        }
    }

    /// Parses an expect statement.
    ///
    /// Format: `expect condition { else -> ... }` or `expect value is pattern { else -> ... }`
    /// or `expect condition { else { ... } }`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::Expect` or an error message.
    fn parse_expect(&mut self) -> Result<Statement, String> {
        self.parser.expect(Token::Expect)?;

        let mut expr_parser = ExpressionParser::new(self.parser);
        let condition = expr_parser.parse_or()?;

        let pattern = if self.parser.current() == &Token::Is {
            self.parser.advance();

            let mut expr_parser = ExpressionParser::new(self.parser);
            let start_expr = expr_parser.parse_or()?;

            if self.parser.current() == &Token::Through {
                self.parser.advance();
                let mut expr_parser = ExpressionParser::new(self.parser);
                let end_expr = expr_parser.parse_or()?;
                Some(ExpectPattern::Range {
                    start: start_expr,
                    end: end_expr,
                    inclusive: true,
                })
            } else if self.parser.current() == &Token::To {
                self.parser.advance();
                let mut expr_parser = ExpressionParser::new(self.parser);
                let end_expr = expr_parser.parse_or()?;
                Some(ExpectPattern::Range {
                    start: start_expr,
                    end: end_expr,
                    inclusive: false,
                })
            } else {
                Some(ExpectPattern::Single(start_expr))
            }
        } else {
            None
        };

        self.parser.expect(Token::LeftBrace)?;
        self.parser.expect(Token::Else)?;

        let else_block = if self.parser.current() == &Token::Arrow {
            self.parser.advance();
            let stmt = self.parse_stmt()?;
            vec![stmt]
        } else if self.parser.current() == &Token::LeftBrace {
            self.parser.advance();
            let mut else_stmts = Vec::new();
            while self.parser.current() != &Token::RightBrace {
                else_stmts.push(self.parse_stmt()?);
            }
            self.parser.expect(Token::RightBrace)?;
            else_stmts
        } else {
            return Err("Expected '->' or '{' after 'else' in expect statement".to_string());
        };

        self.parser.expect(Token::RightBrace)?;

        Ok(Statement::Expect {
            condition,
            pattern,
            else_block,
        })
    }

    /// Parses a next statement (continue).
    ///
    /// Format: `next;`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::Next` or an error message.
    fn parse_next(&mut self) -> Result<Statement, String> {
        self.parser.expect(Token::Next)?;
        self.parser.expect(Token::Semicolon)?;
        Ok(Statement::Next)
    }

    /// Parses a stop statement (break).
    ///
    /// Format: `stop;`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::Stop` or an error message.
    fn parse_stop(&mut self) -> Result<Statement, String> {
        self.parser.expect(Token::Stop)?;
        self.parser.expect(Token::Semicolon)?;
        Ok(Statement::Stop)
    }

    /// Parses a fallthrough statement.
    ///
    /// Format: `fallthrough;`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::Fallthrough` or an error message.
    fn parse_fallthrough(&mut self) -> Result<Statement, String> {
        self.parser.expect(Token::Fallthrough)?;
        self.parser.expect(Token::Semicolon)?;
        Ok(Statement::Fallthrough)
    }

    /// Parses a when statement (switch/match).
    ///
    /// Format: `when value { is pattern { ... } is pattern { ... } else { ... } }`
    /// Or single-line: `when value { is pattern -> statement; }`
    /// Patterns can be:
    /// - Single values: `is 5`
    /// - Ranges: `is 1 to 10` (exclusive) or `is 1 through 10` (inclusive)
    /// - Enum variants: `is Option::Some(x)` or `is Color::Red`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::When` or an error message.
    fn parse_when(&mut self) -> Result<Statement, String> {
        self.parser.expect(Token::When)?;

        let value = match self.parser.current() {
            Token::Identifier(name) => {
                let var_name = name.clone();
                self.parser.advance();
                Expression::Variable(var_name)
            }
            Token::IntLiteral(n) => {
                let num = *n;
                self.parser.advance();
                Expression::IntLiteral(num)
            }
            Token::True => {
                self.parser.advance();
                Expression::BoolLiteral(true)
            }
            Token::False => {
                self.parser.advance();
                Expression::BoolLiteral(false)
            }
            _ => {
                let mut expr_parser = ExpressionParser::new(self.parser);
                expr_parser.parse_or()?
            }
        };

        self.parser.expect(Token::LeftBrace)?;

        let mut cases = Vec::new();

        while self.parser.current() == &Token::Is {
            self.parser.advance();

            let pattern = self.parse_when_pattern()?;

            let case_body = if self.parser.current() == &Token::Arrow {
                self.parser.advance();
                let stmt = self.parse_stmt()?;
                vec![stmt]
            } else {
                self.parser.expect(Token::LeftBrace)?;

                let mut stmts = Vec::new();
                while self.parser.current() != &Token::RightBrace {
                    stmts.push(self.parse_stmt()?);
                }

                self.parser.expect(Token::RightBrace)?;
                stmts
            };

            cases.push(WhenCase {
                pattern,
                body: case_body,
            });
        }

        let else_block = if self.parser.current() == &Token::Else {
            self.parser.advance();

            if self.parser.current() == &Token::Arrow {
                self.parser.advance();
                let stmt = self.parse_stmt()?;
                Some(vec![stmt])
            } else {
                self.parser.expect(Token::LeftBrace)?;
                let mut else_stmts = Vec::new();
                while self.parser.current() != &Token::RightBrace {
                    else_stmts.push(self.parse_stmt()?);
                }
                self.parser.expect(Token::RightBrace)?;
                Some(else_stmts)
            }
        } else {
            None
        };
        
        self.parser.expect(Token::RightBrace)?;
        
        if cases.is_empty() && else_block.is_none() {
            return Err("When statement must have at least one 'is' case or an 'else' block".to_string());
        }

        Ok(Statement::When {
            value,
            cases,
            else_block,
        })
    }


    /// Parses a when pattern (including enum patterns).
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `WhenPattern` or an error message.
    fn parse_when_pattern(&mut self) -> Result<WhenPattern, String> {
        let saved_pos = self.parser.pos;

        if let Token::Identifier(first) = self.parser.current() {
            let first_name = first.clone();
            self.parser.advance();

            if self.parser.current() == &Token::DoubleColon {
                self.parser.advance();

                if let Token::Identifier(variant) = self.parser.current() {
                    let variant_name = variant.clone();
                    self.parser.advance();

                    let bindings = if self.parser.current() == &Token::LeftParen {
                        self.parser.advance();
                        let mut bindings = Vec::new();

                        if self.parser.current() != &Token::RightParen {
                            loop {
                                if let Token::Identifier(binding) = self.parser.current() {
                                    bindings.push(binding.clone());
                                    self.parser.advance();
                                } else {
                                    return Err("Expected variable binding in enum pattern".to_string());
                                }

                                if self.parser.current() == &Token::Comma {
                                    self.parser.advance();
                                } else {
                                    break;
                                }
                            }
                        }

                        self.parser.expect(Token::RightParen)?;
                        bindings
                    } else {
                        vec![]
                    };

                    return Ok(WhenPattern::EnumVariant {
                        enum_name: first_name,
                        variant_name,
                        bindings,
                    });
                }
            }

            self.parser.pos = saved_pos;
        }

        let mut expr_parser = ExpressionParser::new(self.parser);
        let start_expr = expr_parser.parse_or()?;

        if self.parser.current() == &Token::To {
            self.parser.advance();
            let mut expr_parser = ExpressionParser::new(self.parser);
            let end_expr = expr_parser.parse_or()?;
            Ok(WhenPattern::Range {
                start: start_expr,
                end: end_expr,
                inclusive: false,
            })
        } else if self.parser.current() == &Token::Through {
            self.parser.advance();
            let mut expr_parser = ExpressionParser::new(self.parser);
            let end_expr = expr_parser.parse_or()?;
            Ok(WhenPattern::Range {
                start: start_expr,
                end: end_expr,
                inclusive: true,
            })
        } else {
            Ok(WhenPattern::Single(start_expr))
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
            Some(self.parse_type()?)
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
            Some(self.parse_type()?)
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
    /// Format: `var {name}: {type} = {value};`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Statement::Var` or an error message.
    fn parse_var(&mut self) -> Result<Statement, String> {
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
            Some(self.parse_type()?)
        } else {
            None
        };

        self.parser.expect(Token::Assign)?;
        let mut expr_parser = ExpressionParser::new(self.parser);
        let value = expr_parser.parse_expr()?;
        self.parser.expect(Token::Semicolon)?;

        Ok(Statement::Var { name, var_type, value })
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

        let mut elseif_blocks = Vec::new();
        while self.parser.current() == &Token::ElseIf {
            self.parser.advance();
            let mut expr_parser = ExpressionParser::new(self.parser);
            let elseif_condition = expr_parser.parse_expr()?;
            self.parser.expect(Token::LeftBrace)?;

            let mut elseif_body = Vec::new();
            while self.parser.current() != &Token::RightBrace {
                elseif_body.push(self.parse_stmt()?);
            }
            self.parser.expect(Token::RightBrace)?;

            elseif_blocks.push(ElseIfBlock {
                condition: elseif_condition,
                body: elseif_body,
            });
        }

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

        Ok(Statement::If { condition, then_block, elseif_blocks, else_block })
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