use crate::frontend::ast::*;
use std::collections::HashMap;
use super::analyzer::SemanticAnalyzer;
use super::expression_analyzer::ExpressionAnalyzer;

/// Manages variable scopes during semantic analysis.
pub struct ScopeManager<'a> {
    /// The main program analyzer checking for semantic correctness
    analyzer: &'a SemanticAnalyzer,
}

impl<'a> ScopeManager<'a> {
    /// Creates a new scope manager.
    ///
    /// # Parameters
    /// - `analyzer`: Reference to the semantic analyzer
    ///
    /// # Returns
    /// A new ScopeManager instance
    pub fn new(analyzer: &'a SemanticAnalyzer) -> Self {
        ScopeManager { analyzer }
    }

    /// Analyzes an expression within the given scope.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to analyze
    /// - `scope`: Current variable scope mapping names to types
    ///
    /// # Returns
    /// Ok(()) if analysis succeeds, Err with message on failure
    pub fn analyze_expression(&self, expr: &Expression,
                              scope: &HashMap<String, String>) -> Result<(), String> {
        let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
        expr_analyzer.analyze_expr(expr, scope)
    }
}