use crate::frontend::lexer::Token;
use crate::frontend::ast::*;
use super::parser::Parser;
use super::expression_parser::ExpressionParser;

/// Parses struct definitions and struct-related expressions.
pub struct StructParser<'a> {
    parser: &'a mut Parser,
}

impl<'a> StructParser<'a> {
    /// Creates a new StructParser for the given Parser.
    pub fn new(parser: &'a mut Parser) -> Self {
        StructParser { parser }
    }

    /// Parses a struct definition.
    ///
    /// Format: `struct Name { field1: type1, field2: type2 }`
    ///
    /// # Returns
    /// A `StructDef` if successful, or an error message.
    pub fn parse_struct(&mut self) -> Result<StructDef, String> {
        self.parser.expect(Token::Struct)?;

        let name = if let Token::Identifier(n) = self.parser.current() {
            let name = n.clone();
            self.parser.advance();
            name
        } else {
            return Err("Expected struct name".to_string());
        };

        self.parser.expect(Token::LeftBrace)?;

        let mut fields = Vec::new();

        while self.parser.current() != &Token::RightBrace {
            let field_name = if let Token::Identifier(n) = self.parser.current() {
                let name = n.clone();
                self.parser.advance();
                name
            } else {
                return Err("Expected field name".to_string());
            };

            self.parser.expect(Token::Colon)?;
            
            let field_type = match self.parser.current() {
                Token::Type(t) => {
                    let typ = t.clone();
                    self.parser.advance();
                    typ
                }
                Token::Identifier(t) => {
                    let typ = t.clone();
                    self.parser.advance();
                    typ
                }
                _ => return Err(format!("Expected field type, got {:?}", self.parser.current())),
            };

            fields.push(StructField {
                name: field_name,
                field_type,
            });

            if self.parser.current() == &Token::Comma {
                self.parser.advance();
            }
        }

        self.parser.expect(Token::RightBrace)?;

        if fields.is_empty() {
            return Err("Struct must have at least one field".to_string());
        }

        Ok(StructDef { name, fields })
    }

    /// Parses a struct instantiation expression.
    ///
    /// Format: `StructName { field1: value1, field2: value2 }` or `StructName { value1, value2 }`
    ///
    /// # Parameters
    /// - `struct_name`: The name of the struct being instantiated
    ///
    /// # Returns
    /// An `Expression::StructInit` if successful, or an error message.
    pub fn parse_struct_init(&mut self, struct_name: String) -> Result<Expression, String> {
        self.parser.expect(Token::LeftBrace)?;

        let mut fields = Vec::new();
        let mut is_positional = None;

        while self.parser.current() != &Token::RightBrace {
            let is_named_field = if let Token::Identifier(_) = self.parser.current() {
                self.parser.peek(1) == &Token::Colon
            } else {
                false
            };
            
            if is_positional.is_none() {
                is_positional = Some(!is_named_field);
            }

            if is_positional == Some(true) && is_named_field {
                return Err("Cannot mix positional and named field initialization".to_string());
            }
            if is_positional == Some(false) && !is_named_field {
                return Err("Cannot mix named and positional field initialization".to_string());
            }

            let field_init = if is_named_field {
                let field_name = if let Token::Identifier(n) = self.parser.current() {
                    let name = n.clone();
                    self.parser.advance();
                    name
                } else {
                    return Err("Expected field name".to_string());
                };

                self.parser.expect(Token::Colon)?;

                let mut expr_parser = ExpressionParser::new(self.parser);
                let value = expr_parser.parse_expr()?;

                StructFieldInit {
                    name: Some(field_name),
                    value,
                }
            } else {
                let mut expr_parser = ExpressionParser::new(self.parser);
                let value = expr_parser.parse_expr()?;

                StructFieldInit {
                    name: None,
                    value,
                }
            };

            fields.push(field_init);

            if self.parser.current() == &Token::Comma {
                self.parser.advance();
            } else if self.parser.current() != &Token::RightBrace {
                return Err("Expected ',' or '}' after field initialization".to_string());
            }
        }

        self.parser.expect(Token::RightBrace)?;

        if fields.is_empty() {
            return Err("Struct initialization must have at least one field".to_string());
        }

        Ok(Expression::StructInit {
            struct_name,
            fields,
        })
    }
}