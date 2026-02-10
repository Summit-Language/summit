use crate::frontend::ast::*;
use std::collections::HashMap;
use super::type_checker_utils::TypeCheckerUtils;
use crate::utils::io_path_matcher::IoPathMatcher;

/// Performs type inference on expressions.
pub struct TypeInference;

impl TypeInference {
    /// Creates a new TypeInference instance.
    pub fn new() -> Self {
        TypeInference
    }

    /// Infers the type of expression.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to analyze
    /// - `scope`: Current variable scope with type information
    /// - `functions`: Available function definitions
    /// - `type_utils`: Type utility functions
    ///
    /// # Returns
    /// Result containing the inferred type or an error message
    pub fn infer_type(&self, expr: &Expression, scope: &HashMap<String, String>,
                      functions: &HashMap<String, Function>,
                      type_utils: &TypeCheckerUtils) -> Result<String, String> {
        match expr {
            Expression::IntLiteral(val) => {
                self.infer_int_literal_type(*val)
            }
            Expression::BoolLiteral(_) => Ok("bool".to_string()),
            Expression::StringLiteral(_) => Ok("str".to_string()),
            Expression::NullLiteral => Ok("void*".to_string()),
            Expression::Variable(name) => {
                scope.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Expression::Call { path, type_args, .. } => {
                self.infer_call_type(path, type_args, functions)
            }
            Expression::Binary { left, right, op } => {
                self.infer_binary_type(left, right, op, scope, functions, type_utils)
            }
            Expression::Unary { op, operand } => {
                self.infer_unary_type(op, operand, scope, functions, type_utils)
            }
            Expression::IfExpr { then_expr, else_expr, .. } => {
                self.infer_if_expr_type(then_expr, else_expr, scope, functions, type_utils)
            }
            Expression::WhenExpr { cases, else_expr, .. } => {
                self.infer_when_expr_type(cases, else_expr, scope, functions, type_utils)
            }
        }
    }

    /// Infers the smallest type that can hold an integer literal.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `val`: The integer literal value
    ///
    /// # Returns
    /// Result containing the inferred type
    fn infer_int_literal_type(&self, val: u128) -> Result<String, String> {
        if val <= i8::MAX as u128 {
            Ok("i8".to_string())
        } else if val <= u8::MAX as u128 && val <= i16::MAX as u128 {
            Ok("u8".to_string())
        } else if val <= i16::MAX as u128 {
            Ok("i16".to_string())
        } else if val <= u16::MAX as u128 && val <= i32::MAX as u128 {
            Ok("u16".to_string())
        } else if val <= i32::MAX as u128 {
            Ok("i32".to_string())
        } else if val <= u32::MAX as u128 && val <= i64::MAX as u128 {
            Ok("u32".to_string())
        } else if val <= i64::MAX as u128 {
            Ok("i64".to_string())
        } else if val <= u64::MAX as u128 {
            Ok("u64".to_string())
        } else if val <= i128::MAX as u128 {
            Ok("i128".to_string())
        } else {
            Ok("u128".to_string())
        }
    }

    /// Infers the return type of function call.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `path`: The function path
    /// - `type_args`: Generic type arguments if present
    /// - `functions`: Available function definitions
    ///
    /// # Returns
    /// Result containing the inferred return type or an error message
    fn infer_call_type(&self, path: &[String], type_args: &Option<Vec<String>>,
                       functions: &HashMap<String, Function>) -> Result<String, String> {
        if IoPathMatcher::is_readln(path) {
            return Ok("str".to_string());
        }

        if IoPathMatcher::is_read(path) {
            if let Some(types) = type_args {
                if types.len() == 1 {
                    return Ok(types[0].clone());
                }
            }
            return Err("io::read requires a type parameter".to_string());
        }

        if path.len() == 1 {
            if let Some(func) = functions.get(&path[0]) {
                Ok(func.return_type.clone())
            } else {
                Ok("i64".to_string())
            }
        } else {
            Ok("i64".to_string())
        }
    }

    /// Infers the result type of binary operation.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `left`: Left operand expression
    /// - `right`: Right operand expression
    /// - `op`: The binary operator
    /// - `scope`: Current variable scope with type information
    /// - `functions`: Available function definitions
    /// - `type_utils`: Type utility functions
    ///
    /// # Returns
    /// Result containing the inferred type or an error message
    fn infer_binary_type(&self, left: &Expression, right: &Expression, op: &BinaryOp,
                         scope: &HashMap<String, String>, functions: &HashMap<String, Function>,
                         type_utils: &TypeCheckerUtils) -> Result<String, String> {
        match op {
            BinaryOp::Equal | BinaryOp::NotEqual |
            BinaryOp::Less | BinaryOp::Greater |
            BinaryOp::LessEqual | BinaryOp::GreaterEqual |
            BinaryOp::And | BinaryOp::Or => {
                Ok("bool".to_string())
            }
            _ => {
                let left_type = self.infer_type(left, scope, functions, type_utils)?;
                let right_type = self.infer_type(right, scope, functions, type_utils)?;
                Ok(type_utils.wider_type(&left_type, &right_type))
            }
        }
    }

    /// Infers the result type of unary operation.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `op`: The unary operator
    /// - `operand`: The operand expression
    /// - `scope`: Current variable scope with type information
    /// - `functions`: Available function definitions
    /// - `type_utils`: Type utility functions
    ///
    /// # Returns
    /// Result containing the inferred type or an error message
    fn infer_unary_type(&self, op: &UnaryOp, operand: &Expression, scope: &HashMap<String, String>,
                        functions: &HashMap<String, Function>,
                        type_utils: &TypeCheckerUtils) -> Result<String, String> {
        match op {
            UnaryOp::Negate => {
                self.infer_type(operand, scope, functions, type_utils)
            }
            UnaryOp::Not => {
                Ok("bool".to_string())
            }
        }
    }

    /// Infers the result type of if expression.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `then_expr`: The then branch expression
    /// - `else_expr`: The else branch expression
    /// - `scope`: Current variable scope with type information
    /// - `functions`: Available function definitions
    /// - `type_utils`: Type utility functions
    ///
    /// # Returns
    /// Result containing the inferred type or an error message
    fn infer_if_expr_type(&self, then_expr: &Expression, else_expr: &Expression,
                          scope: &HashMap<String, String>, functions: &HashMap<String, Function>,
                          type_utils: &TypeCheckerUtils) -> Result<String, String> {
        let then_type = self.infer_type(then_expr, scope, functions, type_utils)?;
        let else_type = self.infer_type(else_expr, scope, functions, type_utils)?;
        Ok(type_utils.wider_type(&then_type, &else_type))
    }

    /// Infers the result type of when expression.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `cases`: The when cases
    /// - `else_expr`: The else branch expression
    /// - `scope`: Current variable scope with type information
    /// - `functions`: Available function definitions
    /// - `type_utils`: Type utility functions
    ///
    /// # Returns
    /// Result containing the inferred type or an error message
    fn infer_when_expr_type(&self, cases: &[WhenExprCase], else_expr: &Expression,
                            scope: &HashMap<String, String>, functions: &HashMap<String, Function>,
                            type_utils: &TypeCheckerUtils) -> Result<String, String> {
        if cases.is_empty() {
            return self.infer_type(else_expr, scope, functions, type_utils);
        }

        let first_type = self.infer_type(&cases[0].result, scope, functions, type_utils)?;
        
        let mut result_type = first_type;

        for case in &cases[1..] {
            let case_type = self.infer_type(&case.result, scope, functions, type_utils)?;
            result_type = type_utils.wider_type(&result_type, &case_type);
        }

        let else_type = self.infer_type(else_expr, scope, functions, type_utils)?;
        result_type = type_utils.wider_type(&result_type, &else_type);

        Ok(result_type)
    }
}