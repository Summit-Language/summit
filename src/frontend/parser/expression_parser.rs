use crate::frontend::lexer::Token;
use crate::frontend::ast::*;
use super::parser::Parser;

/// Parses expressions in Summit code.
pub struct ExpressionParser<'a> {
    parser: &'a mut Parser,
}

impl<'a> ExpressionParser<'a> {
    /// Creates a new ExpressionParser for the given Parser.
    ///
    /// # Parameters
    /// - `parser`: The parser to use for token handling
    pub fn new(parser: &'a mut Parser) -> Self {
        ExpressionParser { parser }
    }

    /// Parses a full expression from the current token position.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    pub fn parse_expr(&mut self) -> Result<Expression, String> {
        self.parse_ternary()
    }

    /// Parses ternary expressions.
    ///
    /// Format: `condition ? then_expr : else_expr`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    fn parse_ternary(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_when_expr()?;

        if self.parser.current() == &Token::Question {
            self.parser.advance();
            let then_expr = Box::new(self.parse_when_expr()?);
            self.parser.expect(Token::Colon)?;
            let else_expr = Box::new(self.parse_ternary()?);

            expr = Expression::IfExpr {
                condition: Box::new(expr),
                then_expr,
                else_expr,
            };
        }

        Ok(expr)
    }

    /// Parses when expressions.
    ///
    /// Format: `when value { is pattern -> expr, ... else -> expr }`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    fn parse_when_expr(&mut self) -> Result<Expression, String> {
        if self.parser.current() == &Token::When {
            self.parser.advance();

            let value = Box::new(self.parse_or()?);

            self.parser.expect(Token::LeftBrace)?;

            let mut cases = Vec::new();

            while self.parser.current() == &Token::Is {
                self.parser.advance();

                let pattern = self.parse_when_pattern()?;

                self.parser.expect(Token::Arrow)?;

                let result = self.parse_or()?;

                cases.push(WhenExprCase { pattern, result });

                // Optional comma or semicolon separator
                if self.parser.current() == &Token::Comma || self.parser.current() == &Token::Semicolon {
                    self.parser.advance();
                }
            }

            self.parser.expect(Token::Else)?;
            self.parser.expect(Token::Arrow)?;

            let else_expr = Box::new(self.parse_or()?);

            // Optional semicolon after else expression
            if self.parser.current() == &Token::Semicolon {
                self.parser.advance();
            }

            self.parser.expect(Token::RightBrace)?;

            if cases.is_empty() {
                return Err("When expression must have at least one case".to_string());
            }

            Ok(Expression::WhenExpr { value, cases, else_expr })
        } else {
            self.parse_if_expr()
        }
    }

    /// Parses a when pattern (single value or range).
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `WhenPattern` or an error message.
    fn parse_when_pattern(&mut self) -> Result<WhenPattern, String> {
        let start = self.parse_or()?;

        if self.parser.current() == &Token::Through {
            self.parser.advance();
            let end = self.parse_or()?;
            Ok(WhenPattern::Range { start, end, inclusive: true })
        } else {
            Ok(WhenPattern::Single(start))
        }
    }

    /// Parses if expressions.
    ///
    /// Format: `if condition { ... } else { ... }`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    fn parse_if_expr(&mut self) -> Result<Expression, String> {
        if self.parser.current() == &Token::If {
            self.parser.advance();

            let condition = Box::new(self.parse_or()?);

            self.parser.expect(Token::LeftBrace)?;

            let mut then_stmts = Vec::new();
            while self.parser.current() != &Token::RightBrace {
                let mut stmt_parser =
                    super::statement_parser::StatementParser::new(self.parser);
                then_stmts.push(stmt_parser.parse_stmt()?);
            }
            self.parser.expect(Token::RightBrace)?;

            let then_expr = if then_stmts.is_empty() {
                return Err("If expression must have at least one statement in then block"
                    .to_string());
            } else {
                match then_stmts.pop().unwrap() {
                    Statement::Expression(e) => Box::new(e),
                    Statement::Return(e) => Box::new(e),
                    _ => return Err("If expression's then block must end with an expression"
                        .to_string()),
                }
            };

            self.parser.expect(Token::Else)?;
            self.parser.expect(Token::LeftBrace)?;

            let mut else_stmts = Vec::new();
            while self.parser.current() != &Token::RightBrace {
                let mut stmt_parser =
                    super::statement_parser::StatementParser::new(self.parser);
                else_stmts.push(stmt_parser.parse_stmt()?);
            }
            self.parser.expect(Token::RightBrace)?;

            let else_expr = if else_stmts.is_empty() {
                return Err("If expression must have at least one statement in else block"
                    .to_string());
            } else {
                match else_stmts.pop().unwrap() {
                    Statement::Expression(e) => Box::new(e),
                    Statement::Return(e) => Box::new(e),
                    _ => return Err("If expression's else block must end with an expression"
                        .to_string()),
                }
            };

            Ok(Expression::IfExpr { condition, then_expr, else_expr })
        } else {
            self.parse_or()
        }
    }

    /// Parses logical OR expressions.
    ///
    /// Format: `expr or expr`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    pub fn parse_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and()?;

        while self.parser.current() == &Token::Or {
            self.parser.advance();
            let right = self.parse_and()?;
            left = Expression::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parses logical AND expressions.
    ///
    /// Format: `expr and expr`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;

        while self.parser.current() == &Token::And {
            self.parser.advance();
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parses comparison expressions.
    ///
    /// Format: `expr == expr`, `expr < expr`, etc.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_additive()?;

        loop {
            let op = match self.parser.current() {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                Token::LeftAngle => {
                    let next = self.parser.peek(1);
                    let is_generic = match next {
                        Token::Type(_) => {
                            matches!(self.parser.peek(2), Token::RightAngle)
                        }
                        _ => false,
                    };

                    if is_generic {
                        break;
                    } else {
                        BinaryOp::Less
                    }
                }
                Token::RightAngle => BinaryOp::Greater,
                _ => break,
            };
            self.parser.advance();
            let right = self.parse_additive()?;
            left = Expression::Binary { op, left: Box::new(left), right: Box::new(right) };
        }

        Ok(left)
    }

    /// Parses addition and subtraction expressions.
    ///
    /// Format: `expr + expr`, `expr - expr`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    fn parse_additive(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.parser.current() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.parser.advance();
            let right = self.parse_multiplicative()?;
            left = Expression::Binary { op, left: Box::new(left), right: Box::new(right) };
        }

        Ok(left)
    }

    /// Parses multiplication, division, and modulo expressions.
    ///
    /// Format: `expr * expr`, `expr / expr`,
    /// `expr % expr`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    fn parse_multiplicative(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.parser.current() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.parser.advance();
            let right = self.parse_unary()?;
            left = Expression::Binary { op, left: Box::new(left), right: Box::new(right) };
        }

        Ok(left)
    }

    /// Parses unary expressions.
    ///
    /// Format: `-expr`, `not expr`
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    fn parse_unary(&mut self) -> Result<Expression, String> {
        match self.parser.current() {
            Token::Minus => {
                self.parser.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Negate,
                    operand: Box::new(operand),
                })
            }
            Token::Not => {
                self.parser.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_primary(),
        }
    }

    /// Parses primary expressions: literals, variables, function calls, parenthesized expressions.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// An `Expression` or an error message.
    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.parser.current().clone() {
            Token::IntLiteral(n) => {
                self.parser.advance();
                Ok(Expression::IntLiteral(n))
            }
            Token::StringLiteral(s) => {
                self.parser.advance();
                Ok(Expression::StringLiteral(s))
            }
            Token::Null => {
                self.parser.advance();
                Ok(Expression::NullLiteral)
            }
            Token::True => {
                self.parser.advance();
                Ok(Expression::BoolLiteral(true))
            }
            Token::False => {
                self.parser.advance();
                Ok(Expression::BoolLiteral(false))
            }
            Token::Identifier(name) => {
                self.parser.advance();

                let mut path = vec![name];

                while self.parser.current() == &Token::DoubleColon {
                    self.parser.advance();
                    if let Token::Identifier(n) = self.parser.current() {
                        path.push(n.clone());
                        self.parser.advance();
                    } else {
                        return Err("Expected identifier after ::".to_string());
                    }
                }

                let type_args = if self.parser.current() == &Token::LeftAngle
                    && matches!(self.parser.peek(1), Token::Type(_)) {
                    let mut lookahead = 1;
                    let mut is_generic = false;

                    if matches!(self.parser.peek(lookahead), Token::Type(_)) {
                        lookahead += 1;
                        if self.parser.peek(lookahead) == &Token::RightAngle {
                            lookahead += 1;
                            if self.parser.peek(lookahead) == &Token::LeftParen {
                                is_generic = true;
                            }
                        }
                    }

                    if is_generic {
                        self.parser.advance();
                        let mut types = Vec::new();

                        loop {
                            if let Token::Type(t) = self.parser.current() {
                                types.push(t.clone());
                                self.parser.advance();
                            } else {
                                return Err("Expected type in generic parameter".to_string());
                            }

                            if self.parser.current() == &Token::Comma {
                                self.parser.advance();
                            } else {
                                break;
                            }
                        }

                        self.parser.expect(Token::RightAngle)?;
                        Some(types)
                    } else {
                        None
                    }
                } else {
                    None
                };

                if self.parser.current() == &Token::LeftParen {
                    self.parser.advance();
                    let mut args = Vec::new();

                    if self.parser.current() != &Token::RightParen {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.parser.current() == &Token::Comma {
                                self.parser.advance();
                            } else {
                                break;
                            }
                        }
                    }

                    self.parser.expect(Token::RightParen)?;
                    Ok(Expression::Call { path, type_args, args })
                } else if path.len() == 1 && type_args.is_none() {
                    Ok(Expression::Variable(path[0].clone()))
                } else {
                    Err("Path without function call or type parameters without function call"
                        .to_string())
                }
            }
            Token::LeftParen => {
                self.parser.advance();
                let expr = self.parse_expr()?;
                self.parser.expect(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token in expression: {:?}", self.parser.current())),
        }
    }
}