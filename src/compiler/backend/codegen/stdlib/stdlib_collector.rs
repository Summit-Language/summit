use crate::frontend::ast::*;
use crate::utils::type_utils::TypeUtils;
use std::collections::{HashMap, HashSet};

/// Collects standard library functions used in a program.
///
/// Analyzes the AST to identify which standard library functions are called
/// so that appropriate C functions can be included in the generated code.
pub struct StdlibCollector<'a> {
    /// Tracks which standard library functions are called
    used_functions: &'a mut HashSet<String>,

    /// Maintains type information for variables
    symbol_table: &'a mut HashMap<String, String>,
}

impl<'a> StdlibCollector<'a> {
    /// Creates a new collector instance.
    ///
    /// # Parameters
    /// - `used_functions`: Set to store identified function names
    /// - `symbol_table`: Map containing variable type information
    pub fn new(used_functions: &'a mut HashSet<String>,
               symbol_table: &'a mut HashMap<String, String>) -> Self {
        StdlibCollector {
            used_functions,
            symbol_table,
        }
    }

    /// Registers a variable in the symbol table with its type.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `name`: Variable name
    /// - `var_type`: Optional explicit type annotation
    /// - `value`: Initialization expression for type inference
    fn register_variable(&mut self, name: &str, var_type: &Option<String>, value: &Expression) {
        if let Some(t) = var_type {
            self.symbol_table.insert(name.to_string(), t.clone());
        } else {
            let inferred = self.infer_expr_type(value);
            self.symbol_table.insert(name.to_string(), inferred);
        }
    }

    /// Analyzes a function body for standard library calls.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `func`: The function to analyze
    pub fn collect_from_func(&mut self, func: &Function) {
        for stmt in &func.body {
            self.collect_from_stmt(stmt);
        }
    }

    /// Analyzes a statement for standard library calls.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `stmt`: The statement to analyze
    pub fn collect_from_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::When { value, cases, else_block } => {
                self.collect_from_expr(value);
                for case in cases {
                    match &case.pattern {
                        WhenPattern::Single(expr) => {
                            self.collect_from_expr(expr);
                        }
                        WhenPattern::Range { start, end, .. } => {
                            self.collect_from_expr(start);
                            self.collect_from_expr(end);
                        }
                    }
                    for s in &case.body {
                        self.collect_from_stmt(s);
                    }
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.collect_from_stmt(s);
                    }
                }
            }
            Statement::For { start, end, step,
                filter, body, .. } => {
                self.collect_from_expr(start);
                self.collect_from_expr(end);
                if let Some(s) = step {
                    self.collect_from_expr(s);
                }
                if let Some(f) = filter {
                    self.collect_from_expr(f);
                }
                for s in body {
                    self.collect_from_stmt(s);
                }
            }
            Statement::Var { name, var_type, value } => {
                self.register_variable(name, var_type, value);
                self.collect_from_expr(value);
            }
            Statement::Const { name, var_type, value } => {
                self.register_variable(name, var_type, value);
                self.collect_from_expr(value);
            }
            Statement::Comptime { name, var_type, value } => {
                self.register_variable(name, var_type, value);
                self.collect_from_expr(value);
            }
            Statement::Assign { value, .. } => {
                self.collect_from_expr(value);
            }
            Statement::Return(expr) => {
                self.collect_from_expr(expr);
            }
            Statement::Expression(expr) => {
                self.collect_from_expr(expr);
            }
            Statement::If { condition, then_block,
                else_block } => {
                self.collect_from_expr(condition);
                for s in then_block {
                    self.collect_from_stmt(s);
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.collect_from_stmt(s);
                    }
                }
            }
            Statement::While { condition, body } => {
                self.collect_from_expr(condition);
                for s in body {
                    self.collect_from_stmt(s);
                }
            }
            Statement::Next => {}
            Statement::Stop => {}
            Statement::Fallthrough => {}
        }
    }

    /// Analyzes an expression for standard library calls.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `expr`: The expression to analyze
    pub fn collect_from_expr(&mut self, expr: &Expression) {
        match expr {
            Expression::Call { path, type_args,
                args } => {
                self.process_call(path, type_args, args);

                for arg in args {
                    self.collect_from_expr(arg);
                }
            }
            Expression::Binary { left, right, .. } => {
                self.collect_from_expr(left);
                self.collect_from_expr(right);
            }
            Expression::Unary { operand, .. } => {
                self.collect_from_expr(operand);
            }
            Expression::IfExpr { condition, then_expr,
                else_expr } => {
                self.collect_from_expr(condition);
                self.collect_from_expr(then_expr);
                self.collect_from_expr(else_expr);
            }
            _ => {}
        }
    }

    /// Processes a function call to identify standard library usage.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `path`: The function path (e.g., ["io", "println"])
    /// - `type_args`: Generic type arguments if present
    /// - `args`: The function arguments
    fn process_call(&mut self, path: &[String], type_args: &Option<Vec<String>>,
                    args: &[Expression]) {
        let is_stdlib = (path.len() == 2 && path[0] == "io") ||
            (path.len() == 3 && path[0] == "std" && path[1] == "io");

        if !is_stdlib {
            return;
        }

        let func_name = if path.len() == 2 {
            &path[1]
        } else {
            &path[2]
        };

        match func_name.as_str() {
            "readln" => {
                self.used_functions.insert("sm_std_io_readln".to_string());
            }
            "read" => {
                if let Some(types) = type_args {
                    if types.len() == 1 {
                        let func = format!("sm_std_io_read_{}", types[0]);
                        self.used_functions.insert(func);
                    }
                }
            }
            "print" => {
                self.process_print_call(args, false);
            }
            "println" => {
                self.process_println_call(args);
            }
            _ => {}
        }
    }

    /// Processes a `print` function call.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `args`: The arguments to print
    /// - `_is_println`: Whether this is a println call (unused parameter)
    fn process_print_call(&mut self, args: &[Expression], _is_println: bool) {
        if args.len() == 1 {
            if let Expression::StringLiteral(s) = &args[0] {
                if s.contains("{}") {
                    self.used_functions.insert("sm_std_io_print".to_string());
                    for arg in &args[1..] {
                        let arg_type = self.infer_expr_type(arg);
                        self.add_print_function_for_type(&arg_type, false);
                    }
                } else {
                    self.used_functions.insert("sm_std_io_print".to_string());
                }
            } else {
                let arg_type = self.infer_expr_type(&args[0]);
                self.add_print_function_for_type(&arg_type, false);
            }
        } else {
            self.used_functions.insert("sm_std_io_print".to_string());
        }
    }

    /// Processes a `println` function call.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `args`: The arguments to println
    fn process_println_call(&mut self, args: &[Expression]) {
        if args.is_empty() {
            self.used_functions.insert("sm_std_io_println".to_string());
        } else if args.len() == 1 {
            if let Expression::StringLiteral(s) = &args[0] {
                if s.contains("{}") {
                    self.used_functions.insert("sm_std_io_print".to_string());
                    self.used_functions.insert("sm_std_io_println".to_string());
                    for arg in &args[1..] {
                        let arg_type = self.infer_expr_type(arg);
                        self.add_print_function_for_type(&arg_type, false);
                    }
                } else {
                    self.used_functions.insert("sm_std_io_println".to_string());
                }
            } else {
                let arg_type = self.infer_expr_type(&args[0]);
                self.add_print_function_for_type(&arg_type, true);
            }
        } else {
            self.used_functions.insert("sm_std_io_print".to_string());
            self.used_functions.insert("sm_std_io_println".to_string());
            for arg in &args[1..] {
                let arg_type = self.infer_expr_type(arg);
                self.add_print_function_for_type(&arg_type, false);
            }
        }
    }

    /// Adds the appropriate print function for a given type.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `type_name`: The type requiring printing
    /// - `is_println`: Whether the println variant is needed
    fn add_print_function_for_type(&mut self, type_name: &str, is_println: bool) {
        let suffix = if is_println { "ln" } else { "" };

        if type_name == "str" {
            self.used_functions.insert(format!("sm_std_io_print{}", suffix));
        } else if type_name == "bool" {
            self.used_functions.insert(format!("sm_std_io_print{}_bool", suffix));
        } else if type_name.contains("128") {
            if type_name.starts_with('u') {
                self.used_functions.insert(format!("sm_std_io_print{}_u128", suffix));
            } else {
                self.used_functions.insert(format!("sm_std_io_print{}_i128", suffix));
            }
        } else if type_name.starts_with('u') {
            self.used_functions.insert(format!("sm_std_io_print{}_u64", suffix));
        } else if type_name != "str" {
            self.used_functions.insert(format!("sm_std_io_print{}_i64", suffix));
        }
    }

    /// Infers the type of expression.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to analyze
    ///
    /// # Returns
    /// The inferred type as a string
    fn infer_expr_type(&self, expr: &Expression) -> String {
        match expr {
            Expression::IntLiteral(val) => TypeUtils::infer_int_literal_type(*val),
            Expression::BoolLiteral(_) => "bool".to_string(),
            Expression::StringLiteral(_) => "str".to_string(),
            Expression::NullLiteral => "void*".to_string(),
            Expression::Variable(name) => {
                self.symbol_table.get(name).cloned().unwrap_or_else(|| "i64".to_string())
            }
            Expression::IfExpr { then_expr, .. } => {
                self.infer_expr_type(then_expr)
            }
            _ => "i64".to_string(),
        }
    }
}