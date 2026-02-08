use crate::frontend::ast::*;
use std::collections::HashSet;

/// Determines whether expressions can be evaluated at compile time.
pub struct CompileTimeChecker;

impl CompileTimeChecker {
    /// Creates a new CompileTimeChecker instance.
    pub fn new() -> Self {
        CompileTimeChecker
    }

    /// Checks if an expression can be evaluated at compile time.
    ///
    /// Compile time constants are expressions whose values can be
    /// determined without running the program, such as literals,
    /// arithmetic on constants, and references to other constants.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to check
    /// - `runtime_globals`: Set of global variables that require runtime evaluation
    ///
    /// # Returns
    /// `true` if the expression can be evaluated at compile time, `false` otherwise
    pub fn is_compile_time_constant_expr(&self, expr: &Expression,
                                         runtime_globals: &HashSet<String>) -> bool {
        match expr {
            Expression::IntLiteral(_) | Expression::StringLiteral(_) | Expression::BoolLiteral(_)
                | Expression::NullLiteral => true,
            Expression::Variable(name) => {
                !runtime_globals.contains(name)
            }
            Expression::Unary { operand, .. } =>
                self.is_compile_time_constant_expr(operand, runtime_globals),
            Expression::Binary { left, right, .. } => {
                self.is_compile_time_constant_expr(left, runtime_globals) &&
                    self.is_compile_time_constant_expr(right, runtime_globals)
            }
            Expression::IfExpr { condition, then_expr,
                else_expr } => {
                self.is_compile_time_constant_expr(condition, runtime_globals) &&
                    self.is_compile_time_constant_expr(then_expr, runtime_globals) &&
                    self.is_compile_time_constant_expr(else_expr, runtime_globals)
            }
            Expression::Call { .. } => false,
        }
    }
}