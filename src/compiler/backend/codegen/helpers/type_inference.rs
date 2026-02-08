use crate::frontend::ast::*;
use std::collections::HashMap;
use crate::utils::type_utils::TypeUtils;
use crate::utils::io_path_matcher::IoPathMatcher;

/// Infers types for expressions based on context and symbol tables.
pub struct TypeInference<'a> {
    /// Contains type information for variables
    symbol_table: &'a HashMap<String, String>,

    /// Contains return types for functions
    function_signatures: &'a HashMap<String, String>,
}

impl<'a> TypeInference<'a> {
    /// Creates a new TypeInference instance.
    ///
    /// # Parameters
    /// - `symbol_table`: Map of variable names to their types
    /// - `function_signatures`: Map of function names to their return types
    pub fn new(symbol_table: &'a HashMap<String, String>, function_signatures: &'a HashMap<String, String>) -> Self {
        TypeInference {
            symbol_table,
            function_signatures,
        }
    }

    /// Determines the type of expression.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to analyze
    ///
    /// # Returns
    /// The inferred type as a string
    pub fn infer_expression_type(&self, expr: &Expression) -> String {
        match expr {
            Expression::IntLiteral(val) => TypeUtils::infer_int_literal_type(*val),
            Expression::BoolLiteral(_) => "bool".to_string(),
            Expression::Variable(name) => {
                self.symbol_table.get(name).cloned().unwrap_or_else(|| "i64".to_string())
            }
            Expression::Call { path, type_args, .. } => {
                self.infer_call_type(path, type_args)
            }
            Expression::Binary { left, right, .. } => {
                let left_type = self.infer_expression_type(left);
                let right_type = self.infer_expression_type(right);
                self.wider_type(&left_type, &right_type)
            }
            Expression::Unary { operand, .. } => {
                self.infer_expression_type(operand)
            }
            Expression::StringLiteral(_) => "str".to_string(),
            Expression::NullLiteral => "void*".to_string(),
            Expression::IfExpr { then_expr, else_expr, .. } => {
                let then_type = self.infer_expression_type(then_expr);
                let else_type = self.infer_expression_type(else_expr);
                self.wider_type(&then_type, &else_type)
            }
        }
    }

    /// Determines the return type of function call.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `path`: The function path (e.g., ["io", "println"])
    /// - `type_args`: Generic type arguments if present
    ///
    /// # Returns
    /// The inferred return type as a string
    fn infer_call_type(&self, path: &[String], type_args: &Option<Vec<String>>) -> String {
        // Check for readln
        if IoPathMatcher::is_readln(path) {
            return "str".to_string();
        }

        // Check for generic read
        if IoPathMatcher::is_read(path) {
            if let Some(types) = type_args {
                if types.len() == 1 {
                    return types[0].clone();
                }
            }
        }

        if path.len() == 1 {
            self.function_signatures.get(&path[0]).cloned().unwrap_or_else(|| "i64".to_string())
        } else {
            "i64".to_string()
        }
    }

    /// Returns the wider of two numeric types.
    ///
    /// Used for binary operations where the result type should be
    /// the larger of the two operand types (type promotion).
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type1`: The first type
    /// - `type2`: The second type
    ///
    /// # Returns
    /// The wider type as a string
    pub fn wider_type(&self, type1: &str, type2: &str) -> String {
        let type_priority = [
            "bool", "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "i128", "u128"
        ];

        let pos1 = type_priority.iter().position(|&t| t == type1);
        let pos2 = type_priority.iter().position(|&t| t == type2);

        match (pos1, pos2) {
            (Some(p1), Some(p2)) => {
                if p1 > p2 {
                    type1.to_string()
                } else {
                    type2.to_string()
                }
            }
            _ => type1.to_string(),
        }
    }
}