use crate::frontend::ast::*;
use super::program_generator::ProgramGenerator;
use super::super::helpers::TypeInference;
use crate::utils::io_path_matcher::IoPathMatcher;

/// Generates C code for expressions.
pub struct ExpressionGenerator<'a> {
    /// The Program Generator responsible for generating the executable program
    generator: &'a mut ProgramGenerator,
}

impl<'a> ExpressionGenerator<'a> {
    /// Creates a new ExpressionGenerator instance.
    ///
    /// # Parameters
    /// - `generator`: The parent program generator
    pub fn new(generator: &'a mut ProgramGenerator) -> Self {
        ExpressionGenerator { generator }
    }

    /// Generates C code for an expression.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `expr`: The expression to generate code for
    pub fn generate_expr(&mut self, expr: &Expression) {
        match expr {
            Expression::IntLiteral(n) => self.generator.emitter.emit_int_literal(*n),
            Expression::BoolLiteral(b) => self.generator.emitter.emit_bool_literal(*b),
            Expression::StringLiteral(s) => self.generator.emitter
                .emit_string_literal(s),
            Expression::NullLiteral => self.generator.emitter.emit("NULL"),
            Expression::Variable(name) => self.generator.emitter.emit(name),
            Expression::Call { path, type_args,
                args } => self.emit_call(path, type_args, args),
            Expression::Binary { op, left,
                right } => self.emit_binary_op(op, left, right),
            Expression::Unary { op, operand } => self
                .emit_unary_op(op, operand),
            Expression::IfExpr { condition, then_expr,
                else_expr } => {
                self.emit_if_expr(condition, then_expr, else_expr);
            }
        }
    }

    /// Emits C code for a ternary expression.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `condition`: The condition expression
    /// - `then_expr`: The then branch expression
    /// - `else_expr`: The else branch expression
    fn emit_if_expr(&mut self, condition: &Expression, then_expr: &Expression,
                    else_expr: &Expression) {
        self.generator.emitter.emit("(");
        self.generate_expr(condition);
        self.generator.emitter.emit(" ? ");
        self.generate_expr(then_expr);
        self.generator.emitter.emit(" : ");
        self.generate_expr(else_expr);
        self.generator.emitter.emit(")");
    }

    /// Emits C code for a function call.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `path`: The function path
    /// - `type_args`: Generic type arguments
    /// - `args`: Function arguments
    fn emit_call(&mut self, path: &[String], type_args: &Option<Vec<String>>,
                 args: &[Expression]) {
        if IoPathMatcher::is_readln(path) {
            self.generator.emitter.emit("sm_std_io_readln()");
            return;
        }

        if IoPathMatcher::is_read(path) {
            if let Some(types) = type_args {
                if types.len() == 1 {
                    let func_name = format!("sm_std_io_read_{}", types[0]);
                    self.generator.emitter.emit(&func_name);
                    self.generator.emitter.emit("()");
                    return;
                }
            }
            self.generator.emitter.emit("/* Error: read requires a type parameter */");
            return;
        }

        let is_println = IoPathMatcher::is_println(path);
        let is_print = IoPathMatcher::is_print(path);

        if (is_println || is_print) && !args.is_empty() {
            if let Expression::StringLiteral(format_str) = &args[0] {
                if format_str.contains("{}") {
                    self.emit_formatted_print(format_str, &args[1..], is_println);
                    return;
                }
            }
        }

        let func_name = self.resolve_func_name(path, args);
        self.generator.emitter.emit(&func_name);
        self.generator.emitter.emit("(");

        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.generator.emitter.emit(", ");
            }
            self.generate_expr(arg);
        }

        self.generator.emitter.emit(")");
    }

    /// Emits formatted print statements for string interpolation.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `format_str`: The format string containing {} placeholders
    /// - `args`: The arguments to insert into the format string
    /// - `with_newline`: Whether to append a newline after printing
    fn emit_formatted_print(&mut self, format_str: &str, args: &[Expression], with_newline: bool) {
        let parts: Vec<&str> = format_str.split("{}").collect();

        if parts.len() - 1 != args.len() {
            self.generator.emitter.emit("sm_std_io_print");
            if with_newline {
                self.generator.emitter.emit("ln");
            }
            self.generator.emitter.emit("(");
            self.generator.emitter.emit_string_literal(format_str);
            self.generator.emitter.emit(")");
            return;
        }
        
        let type_inference = TypeInference::new(&self.generator.symbol_table,
                                                &self.generator.function_signatures);
        let arg_types: Vec<String> = args.iter()
            .map(|arg| type_inference.infer_expression_type(arg))
            .collect();
        drop(type_inference);

        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                self.generator.emitter.emit("sm_std_io_print(");
                self.generator.emitter.emit_string_literal(part);
                self.generator.emitter.emit(")");
                if i < args.len() || with_newline {
                    self.generator.emitter.emit("; ");
                }
            }

            if i < args.len() {
                let arg_type = &arg_types[i];

                if arg_type == "str" {
                    self.generator.emitter.emit("sm_std_io_print(");
                    self.generate_expr(&args[i]);
                    self.generator.emitter.emit(")");
                } else if arg_type == "bool" {
                    self.generator.emitter.emit("sm_std_io_print_bool(");
                    self.generate_expr(&args[i]);
                    self.generator.emitter.emit(")");
                } else if arg_type.contains("128") {
                    if arg_type.starts_with('u') {
                        self.generator.emitter.emit("sm_std_io_print_u128(");
                    } else {
                        self.generator.emitter.emit("sm_std_io_print_i128(");
                    }
                    self.generate_expr(&args[i]);
                    self.generator.emitter.emit(")");
                } else if arg_type.starts_with('u') {
                    self.generator.emitter.emit("sm_std_io_print_u64(");
                    self.generate_expr(&args[i]);
                    self.generator.emitter.emit(")");
                } else {
                    self.generator.emitter.emit("sm_std_io_print_i64(");
                    self.generate_expr(&args[i]);
                    self.generator.emitter.emit(")");
                }

                if i < parts.len() - 1 || with_newline {
                    self.generator.emitter.emit("; ");
                }
            }
        }

        if with_newline {
            self.generator.emitter.emit("sm_std_io_println(\"\")");
        }
    }

    /// Resolves a function name to its C equivalent.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `path`: The function path
    /// - `args`: The function arguments
    ///
    /// # Returns
    /// The resolved C function name
    fn resolve_func_name(&self, path: &[String], args: &[Expression]) -> String {
        let is_io_call = IoPathMatcher::is_print(path) || IoPathMatcher::is_println(path);

        let mut func_name = if path.len() >= 2 {
            self.build_module_func_name(path)
        } else {
            path[0].clone()
        };

        if is_io_call && args.len() == 1 {
            self.add_io_type_suffix(&mut func_name, &args[0]);
        }

        func_name
    }

    /// Builds a C function name from a module path.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `path`: The module path components
    ///
    /// # Returns
    /// The constructed C function name
    fn build_module_func_name(&self, path: &[String]) -> String {
        if path[0] == "io" {
            vec!["sm", "std", "io", &path[1]].join("_")
        } else if path[0] == "std" && path.len() == 3 {
            vec!["sm", &path[0], &path[1], &path[2]].join("_")
        } else if path[0] == "std" && path.len() == 2 {
            vec!["sm", &path[0], &path[1]].join("_")
        } else {
            vec!["sm", &path.join("_")].join("_")
        }
    }

    /// Adds a type suffix to I/O function names.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `func_name`: The base function name
    /// - `arg`: The argument used to determine the type suffix
    fn add_io_type_suffix(&self, func_name: &mut String, arg: &Expression) {
        let type_inference = TypeInference::new(&self.generator.symbol_table,
                                                &self.generator.function_signatures);
        let expr_type = type_inference.infer_expression_type(arg);
        drop(type_inference);

        if expr_type == "bool" {
            func_name.push_str("_bool");
            return;
        }

        if expr_type == "str" {
            return;
        }

        match expr_type.as_str() {
            "i8" => func_name.push_str("_i64"),
            "u8" => func_name.push_str("_u64"),
            "i16" => func_name.push_str("_i64"),
            "u16" => func_name.push_str("_u64"),
            "i32" => func_name.push_str("_i64"),
            "u32" => func_name.push_str("_u64"),
            "i64" => func_name.push_str("_i64"),
            "u64" => func_name.push_str("_u64"),
            "i128" => func_name.push_str("_i128"),
            "u128" => func_name.push_str("_u128"),
            _ => func_name.push_str("_i64"),
        }
    }

    /// Emits C code for a binary operation.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `op`: The binary operator
    /// - `left`: The left operand
    /// - `right`: The right operand
    fn emit_binary_op(&mut self, op: &BinaryOp, left: &Expression, right: &Expression) {
        self.generator.emitter.emit("(");
        self.generate_expr(left);
        let op_str = self.generator.emitter.emit_binary_op(op);
        self.generator.emitter.emit(&format!(" {} ", op_str));
        self.generate_expr(right);
        self.generator.emitter.emit(")");
    }

    /// Emits C code for a unary operation.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `op`: The unary operator
    /// - `operand`: The operand
    fn emit_unary_op(&mut self, op: &UnaryOp, operand: &Expression) {
        match op {
            UnaryOp::Negate => {
                if let Expression::IntLiteral(n) = operand {
                    if *n == 170141183460469231731687303715884105728u128 {
                        let high = 9223372036854775808u64;
                        self.generator.emitter
                            .emit(&format!("((__int128){}ULL << 64)", high));
                        return;
                    }
                }

                self.generator.emitter.emit("(-");
                self.generate_expr(operand);
                self.generator.emitter.emit(")");
            }
            UnaryOp::Not => {
                self.generator.emitter.emit("(!");
                self.generate_expr(operand);
                self.generator.emitter.emit(")");
            }
        }
    }
}