use crate::frontend::ast::*;
use super::analyzer::SemanticAnalyzer;

/// Validates compile-time evaluation constraints for expressions.
pub struct CompileTimeChecker<'a> {
    /// The main program analyzer checking for semantic correctness
    analyzer: &'a SemanticAnalyzer,
}

impl<'a> CompileTimeChecker<'a> {
    /// Creates a new compile-time checker.
    ///
    /// # Parameters
    /// - `analyzer`: Reference to the semantic analyzer
    ///
    /// # Returns
    /// A new CompileTimeChecker instance
    pub fn new(analyzer: &'a SemanticAnalyzer) -> Self {
        CompileTimeChecker { analyzer }
    }

    /// Checks if an expression is a compile-time constant.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to check
    ///
    /// # Returns
    /// True if the expression is a compile-time constant
    pub fn is_compile_time_constant(&self, expr: &Expression) -> bool {
        match expr {
            Expression::IntLiteral(_) | Expression::StringLiteral(_)
            | Expression::BoolLiteral(_) | Expression::NullLiteral => true,
            Expression::Variable(name) => {
                self.analyzer.global_scope.contains_key(name)
            }
            Expression::Unary { operand, .. } => self
                .is_compile_time_constant(operand),
            Expression::Binary { left, right, .. } => {
                self.is_compile_time_constant(left) && self.is_compile_time_constant(right)
            }
            Expression::IfExpr { condition, then_expr,
                else_expr } => {
                self.is_compile_time_constant(condition) &&
                    self.is_compile_time_constant(then_expr) &&
                    self.is_compile_time_constant(else_expr)
            }
            _ => false,
        }
    }

    /// Checks if an expression can be evaluated at compile time.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to check
    ///
    /// # Returns
    /// True if the expression can be evaluated at compile time
    pub fn is_compile_time_evaluable(&self, expr: &Expression) -> bool {
        match expr {
            Expression::IntLiteral(_) | Expression::StringLiteral(_) |
            Expression::BoolLiteral(_) | Expression::NullLiteral => true,
            Expression::Variable(_) => true,
            Expression::Unary { operand, .. } => self
                .is_compile_time_evaluable(operand),
            Expression::Binary { left, right, .. } => {
                self.is_compile_time_evaluable(left) && self.is_compile_time_evaluable(right)
            }
            Expression::IfExpr { condition, then_expr,
                else_expr } => {
                self.is_compile_time_evaluable(condition) &&
                    self.is_compile_time_evaluable(then_expr) &&
                    self.is_compile_time_evaluable(else_expr)
            }
            Expression::Call { .. } => false,
        }
    }
}