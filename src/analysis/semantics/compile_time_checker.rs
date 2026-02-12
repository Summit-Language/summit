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
            Expression::WhenExpr { value, cases, else_expr } => {
                if !self.is_compile_time_constant(value) {
                    return false;
                }

                for case in cases {
                    let pattern_is_constant = match &case.pattern {
                        WhenPattern::Single(pattern_expr) => {
                            self.is_compile_time_constant(pattern_expr)
                        }
                        WhenPattern::Range { start, end, .. } => {
                            self.is_compile_time_constant(start) &&
                                self.is_compile_time_constant(end)
                        }
                    };

                    if !pattern_is_constant {
                        return false;
                    }

                    if !self.is_compile_time_constant(&case.result) {
                        return false;
                    }
                }

                self.is_compile_time_constant(else_expr)
            }
            Expression::StructInit { fields, .. } => {
                for field_init in fields {
                    if !self.is_compile_time_constant(&field_init.value) {
                        return false;
                    }
                }
                true
            }
            Expression::FieldAccess { object, .. } => {
                self.is_compile_time_constant(object)
            }
            Expression::Call { .. } => false,
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
            Expression::WhenExpr { value, cases, else_expr } => {
                if !self.is_compile_time_evaluable(value) {
                    return false;
                }

                for case in cases {
                    let pattern_is_evaluable = match &case.pattern {
                        WhenPattern::Single(pattern_expr) => {
                            self.is_compile_time_evaluable(pattern_expr)
                        }
                        WhenPattern::Range { start, end, .. } => {
                            self.is_compile_time_evaluable(start) &&
                                self.is_compile_time_evaluable(end)
                        }
                    };

                    if !pattern_is_evaluable {
                        return false;
                    }

                    if !self.is_compile_time_evaluable(&case.result) {
                        return false;
                    }
                }

                self.is_compile_time_evaluable(else_expr)
            }
            Expression::StructInit { fields, .. } => {
                for field_init in fields {
                    if !self.is_compile_time_evaluable(&field_init.value) {
                        return false;
                    }
                }
                true
            }
            Expression::FieldAccess { object, .. } => {
                self.is_compile_time_evaluable(object)
            }
            Expression::Call { .. } => false,
        }
    }
}