use crate::frontend::ast::*;
use std::collections::HashMap;
use super::analyzer::SemanticAnalyzer;
use super::module_checker::ModuleChecker;
use crate::utils::io_path_matcher::IoPathMatcher;

/// Analyzes expressions for semantic correctness.
pub struct ExpressionAnalyzer<'a> {
    /// The main program analyzer checking for semantic correctness
    analyzer: &'a SemanticAnalyzer,
}

impl<'a> ExpressionAnalyzer<'a> {
    /// Creates a new expression analyzer.
    ///
    /// # Parameters
    /// - `analyzer`: Reference to the semantic analyzer
    ///
    /// # Returns
    /// A new ExpressionAnalyzer instance
    pub fn new(analyzer: &'a SemanticAnalyzer) -> Self {
        ExpressionAnalyzer { analyzer }
    }

    /// Analyzes an expression for semantic correctness.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to analyze
    /// - `scope`: Current variable scope mapping names to types
    ///
    /// # Returns
    /// Ok(()) if analysis succeeds, Err with message on failure
    pub fn analyze_expr(&self, expr: &Expression,
                        scope: &HashMap<String, String>) -> Result<(), String> {
        match expr {
            Expression::IntLiteral(_) | Expression::StringLiteral(_) | Expression::BoolLiteral(_)
            | Expression::NullLiteral => Ok(()),
            Expression::Variable(name) => {
                if !scope.contains_key(name) {
                    Err(format!("Undefined variable: {}", name))
                } else {
                    Ok(())
                }
            }
            Expression::Call { path, type_args,
                args } => {
                let module_checker = ModuleChecker::new(self.analyzer);

                if IoPathMatcher::is_read(path) {
                    if type_args.is_none() {
                        return Err("io::read requires a type parameter, e.g., io::read<i32>()"
                            .to_string());
                    }

                    if let Some(types) = type_args {
                        if types.len() != 1 {
                            return Err("io::read requires exactly one type parameter".to_string());
                        }

                        let type_name = &types[0];
                        if !module_checker.is_valid_read_type(type_name) {
                            return Err(format!("io::read does not support type '{}'. Supported types: i8, u8, i16, u16, i32, u32, i64, u64", type_name));
                        }
                    }

                    if !args.is_empty() {
                        return Err("io::read takes no arguments".to_string());
                    }

                    return Ok(());
                }

                if IoPathMatcher::is_readln(path) {
                    if type_args.is_some() {
                        return Err("io::readln does not take type parameters".to_string());
                    }

                    if !args.is_empty() {
                        return Err("io::readln takes no arguments".to_string());
                    }

                    return Ok(());
                }

                if path.len() >= 2 {
                    module_checker.check_module_import(path)?;
                } else if path.len() == 1 {
                    self.check_local_func_call(path, args, scope)?;
                }

                for arg in args {
                    self.analyze_expr(arg, scope)?;
                }
                Ok(())
            }
            Expression::Binary { left, right, .. } => {
                self.analyze_expr(left, scope)?;
                self.analyze_expr(right, scope)?;

                let left_type = self.analyzer.type_checker.infer_type(left, scope,
                                                                      &self.analyzer.functions)?;
                let right_type = self.analyzer.type_checker.infer_type(right, scope,
                                                                       &self.analyzer.functions)?;

                if !self.analyzer.type_checker.types_compatible(&left_type, &right_type) {
                    return Err(format!("Type mismatch in binary operation: {} and {}",
                                       left_type, right_type));
                }

                Ok(())
            }
            Expression::Unary { operand, .. } => {
                self.analyze_expr(operand, scope)
            }
            Expression::IfExpr { condition, then_expr,
                else_expr } => {
                self.analyze_expr(condition, scope)?;

                self.analyze_expr(then_expr, scope)?;
                self.analyze_expr(else_expr, scope)?;

                let then_type = self.analyzer.type_checker.infer_type(then_expr, scope,
                                                                      &self.analyzer.functions)?;
                let else_type = self.analyzer.type_checker.infer_type(else_expr, scope,
                                                                      &self.analyzer.functions)?;

                if !self.analyzer.type_checker.types_compatible(&then_type, &else_type) {
                    return Err(format!(
                        "If expression branches have incompatible types: '{}' and '{}'",
                        then_type, else_type
                    ));
                }

                Ok(())
            }
        }
    }

    /// Validates a local function call.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `path`: Function path
    /// - `args`: Function arguments
    /// - `scope`: Current variable scope
    ///
    /// # Returns
    /// Ok(()) if the call is valid, Err with message on failure
    fn check_local_func_call(&self, path: &[String], args: &[Expression],
                             scope: &HashMap<String, String>) -> Result<(), String> {
        let func_name = &path[0];

        if !self.analyzer.functions.contains_key(func_name) {
            return Err(format!("Undefined function: '{}'", func_name));
        }

        if let Some(params) = self.analyzer.function_params.get(func_name) {
            if args.len() != params.len() {
                return Err(format!(
                    "Function '{}' expects {} arguments, but {} were provided",
                    func_name, params.len(), args.len()
                ));
            }

            for (i, (arg, (param_name, param_type))) in args
                .iter().zip(params.iter()).enumerate() {
                let arg_type = self.analyzer.type_checker.infer_type(arg, scope,
                                                                     &self.analyzer.functions)?;
                if arg_type == *param_type {
                    continue;
                }

                if self.analyzer.type_checker.can_convert(&arg_type, param_type) {
                    continue;
                }

                if self.analyzer.type_checker.is_signed(&arg_type)
                    && !self.analyzer.type_checker.is_signed(param_type) {
                    let arg_size = self.analyzer.type_checker.get_type_size(&arg_type);
                    let param_size = self.analyzer.type_checker.get_type_size(param_type);

                    if arg_size < param_size {
                        if let Some(val) = self.analyzer.type_checker.get_literal_value(arg) {
                            if val > self.analyzer.type_checker.get_unsigned_max(param_type) {
                                return Err(format!(
                                    "Cannot convert value {} to unsigned type '{}' (exceeds maximum: {})",
                                    val, param_type, self.analyzer.type_checker
                                        .get_unsigned_max(param_type)
                                ));
                            }
                        }
                        continue;
                    }
                }

                if !self.analyzer.type_checker.is_signed(&arg_type)
                    && self.analyzer.type_checker.is_signed(param_type) {
                    let arg_size = self.analyzer.type_checker.get_type_size(&arg_type);
                    let param_size = self.analyzer.type_checker.get_type_size(param_type);

                    if arg_size < param_size {
                        continue;
                    }
                }

                if self.analyzer.type_checker.would_truncate(&arg_type, param_type) {
                    let error_msg = match arg {
                        Expression::Variable(var_name) => {
                            format!(
                                "Type mismatch in argument {} of function '{}': variable '{}' has inferred type '{}', but parameter '{}' expects type '{}'. The value assigned to '{}' requires type '{}', which exceeds the range of '{}'",
                                i + 1, func_name, var_name, arg_type, param_name,
                                param_type, var_name, arg_type, param_type
                            )
                        }
                        Expression::IntLiteral(val) => {
                            format!(
                                "Type mismatch in argument {} of function '{}': literal value {} exceeds maximum value for parameter '{}' of type '{}' (maximum: {})",
                                i + 1, func_name, val, param_name, param_type,
                                self.analyzer.type_checker.get_type_max(param_type)
                            )
                        }
                        _ => {
                            format!(
                                "Type mismatch in argument {} of function '{}': expression has type '{}' but parameter '{}' expects type '{}' (would lose data)",
                                i + 1, func_name, arg_type, param_name, param_type
                            )
                        }
                    };
                    return Err(error_msg);
                }

                if !self.analyzer.type_checker.types_compatible(&arg_type, param_type) {
                    return Err(format!(
                        "Type mismatch in argument {} of function '{}': expected '{}', got '{}'",
                        i + 1, func_name, param_type, arg_type
                    ));
                }

                return Err(format!(
                    "Type mismatch in argument {} of function '{}': cannot implicitly convert '{}' to '{}' (same size but different signedness)",
                    i + 1, func_name, arg_type, param_type
                ));
            }
        }

        Ok(())
    }
}