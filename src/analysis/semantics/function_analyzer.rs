use crate::frontend::ast::*;
use std::collections::{HashMap, HashSet};
use super::analyzer::SemanticAnalyzer;
use super::mutation_checker::MutationChecker;
use super::statement_analyzer::StatementAnalyzer;

/// Analyzes function definitions for semantic correctness.
pub struct FunctionAnalyzer<'a> {
    /// The main program analyzer checking for semantic correctness
    analyzer: &'a SemanticAnalyzer,
}

impl<'a> FunctionAnalyzer<'a> {
    /// Creates a new function analyzer.
    ///
    /// # Parameters
    /// - `analyzer`: Reference to the semantic analyzer
    ///
    /// # Returns
    /// A new FunctionAnalyzer instance
    pub fn new(analyzer: &'a SemanticAnalyzer) -> Self {
        FunctionAnalyzer { analyzer }
    }

    /// Analyzes a function definition.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `func`: The function to analyze
    ///
    /// # Returns
    /// Ok(()) if analysis succeeds, Err with message on failure
    pub fn analyze_func(&self, func: &Function) -> Result<(), String> {
        let mut scope = self.analyzer.global_scope.clone();
        let mut mutability = self.analyzer.global_mutability.clone();

        // Add function parameters to scope - parameters are immutable
        for param in &func.params {
            scope.insert(param.name.clone(), param.param_type.clone());
            mutability.insert(param.name.clone(), false);
        }

        let mut mutations = HashSet::new();
        let mut var_declarations = HashMap::new();

        let mutation_checker = MutationChecker::new();
        for stmt in &func.body {
            mutation_checker.collect_mutations(stmt, &mut mutations, &mut var_declarations)?;
        }

        for (var_name, is_var) in &var_declarations {
            if *is_var && !mutations.contains(var_name) {
                return Err(format!(
                    "Variable '{}' is never mutated. Consider using 'const' instead of 'var'",
                    var_name
                ));
            }
        }

        let stmt_analyzer = StatementAnalyzer::new(self.analyzer);
        for stmt in &func.body {
            stmt_analyzer.analyze_stmt_with_return_type(stmt, &mut scope,
                                                        &func.name, &func.return_type,
                                                        &mut mutability)?;
        }

        Ok(())
    }
}