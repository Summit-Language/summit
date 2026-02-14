use crate::frontend::lexer::Token;
use crate::frontend::ast::*;
use super::parser::Parser;
use super::expression_parser::ExpressionParser;
use super::statement_parser::StatementParser;
use super::struct_parser::StructParser;

/// Parses declarations at the global/root level of a Summit program.
pub struct DeclarationParser<'a> {
    parser: &'a mut Parser,
}

impl<'a> DeclarationParser<'a> {
    /// Creates a new DeclarationParser for the given Parser.
    pub fn new(parser: &'a mut Parser) -> Self {
        DeclarationParser { parser }
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

    /// Parses a struct definition.
    ///
    /// Format: `struct Name { field: type }`
    ///
    /// # Returns
    /// A `GlobalDeclaration::Struct` if successful, or an error message.
    pub fn parse_struct(&mut self) -> Result<GlobalDeclaration, String> {
        let mut struct_parser = StructParser::new(self.parser);
        let struct_def = struct_parser.parse_struct()?;
        Ok(GlobalDeclaration::Struct(struct_def))
    }

    /// Parses an enum definition.
    ///
    /// Format: `enum Name { Variant1(type1), Variant2, Variant3(type1, type2) }`
    ///
    /// # Returns
    /// A `GlobalDeclaration::Enum` if successful, or an error message.
    pub fn parse_enum(&mut self) -> Result<GlobalDeclaration, String> {
        self.parser.expect(Token::Enum)?;

        let name = if let Token::Identifier(n) = self.parser.current() {
            let name = n.clone();
            self.parser.advance();
            name
        } else {
            return Err("Expected enum name".to_string());
        };

        self.parser.expect(Token::LeftBrace)?;

        let mut variants = Vec::new();

        while self.parser.current() != &Token::RightBrace {
            if let Token::Identifier(variant_name) = self.parser.current() {
                let variant_name = variant_name.clone();
                self.parser.advance();

                let payload = if self.parser.current() == &Token::LeftParen {
                    self.parser.advance();

                    let mut types = Vec::new();

                    if self.parser.current() != &Token::RightParen {
                        loop {
                            types.push(self.parse_type()?);

                            if self.parser.current() == &Token::Comma {
                                self.parser.advance();
                            } else {
                                break;
                            }
                        }
                    }

                    self.parser.expect(Token::RightParen)?;
                    Some(types)
                } else {
                    None
                };

                variants.push(EnumVariant {
                    name: variant_name,
                    payload,
                });

                if self.parser.current() == &Token::Comma {
                    self.parser.advance();
                } else if self.parser.current() != &Token::RightBrace {
                    return Err("Expected ',' or '}' after enum variant".to_string());
                }
            } else {
                return Err("Expected variant name".to_string());
            }
        }

        self.parser.expect(Token::RightBrace)?;

        if variants.is_empty() {
            return Err("Enum must have at least one variant".to_string());
        }

        Ok(GlobalDeclaration::Enum(EnumDef { name, variants }))
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
            Some(self.parse_type()?)
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
            Some(self.parse_type()?)
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
            Some(self.parse_type()?)
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
    /// Formats:
    /// - `func name(param: type): return_type { ... }`
    /// - `abi "C" func name(param: type): return_type;` (external declaration)
    /// - `abi "C" func printf(fmt: str, args...);` (with varargs)
    ///
    /// # Parameters:
    /// - `self`: Mutable reference to self
    ///
    /// # Returns
    /// A `Function` struct with the function details, or an error message.
    pub fn parse_func(&mut self) -> Result<Function, String> {
        let abi = if matches!(self.parser.current(), Token::Extern) {
            self.parser.advance();

            if let Token::StringLiteral(abi_name) = self.parser.current() {
                let abi = abi_name.clone();
                self.parser.advance();
                Some(abi)
            } else {
                return Err("Expected ABI name as string literal (e.g., \"C\")".to_string());
            }
        } else {
            None
        };

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
        let mut has_varargs = false;

        if self.parser.current() != &Token::RightParen {
            loop {
                if let Token::Identifier(param_name) = self.parser.current() {
                    let param_name_clone = param_name.clone();
                    self.parser.advance();

                    if matches!(self.parser.current(), Token::Ellipsis) {
                        self.parser.advance();
                        has_varargs = true;
                        break;
                    }

                    self.parser.expect(Token::Colon)?;
                    let param_type = self.parse_type()?;

                    params.push(Parameter {
                        name: param_name_clone,
                        param_type
                    });

                    if self.parser.current() == &Token::Comma {
                        self.parser.advance();
                    } else {
                        break;
                    }
                } else {
                    return Err("Expected parameter name".to_string());
                }
            }
        }

        self.parser.expect(Token::RightParen)?;
        self.parser.expect(Token::Colon)?;

        let return_type = self.parse_type()?;

        let body = if self.parser.current() == &Token::Semicolon {
            self.parser.advance();
            Vec::new()
        } else {
            self.parser.expect(Token::LeftBrace)?;

            let mut body = Vec::new();
            while self.parser.current() != &Token::RightBrace {
                let mut stmt_parser = StatementParser::new(self.parser);
                body.push(stmt_parser.parse_stmt()?);
            }

            self.parser.expect(Token::RightBrace)?;
            body
        };

        Ok(Function {
            name,
            params,
            has_varargs,
            return_type,
            abi,
            body
        })
    }
}