use crate::frontend::ast::*;
use std::collections::{HashMap, HashSet};

/// Tracks variable mutations and declarations throughout the AST.
pub struct MutationChecker;

impl MutationChecker {
    /// Creates a new mutation checker.
    ///
    /// # Returns
    /// A new MutationChecker instance
    pub fn new() -> Self {
        MutationChecker
    }

    /// Collects mutations and variable declarations from a statement.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `stmt`: The statement to analyze
    /// - `mutations`: Set to collect variable names that are assigned to
    /// - `var_declarations`: Map of variable names to mutability
    ///
    /// # Returns
    /// Ok(()) if collection succeeds, Err with message on failure
    pub fn collect_mutations(&self, stmt: &Statement, mutations: &mut HashSet<String>,
                             var_declarations: &mut HashMap<String, bool>) -> Result<(), String> {
        match stmt {
            Statement::Let { name, .. } => {
                var_declarations.insert(name.clone(), true);
                Ok(())
            }
            Statement::Const { name, .. } => {
                var_declarations.insert(name.clone(), false);
                Ok(())
            }
            Statement::Comptime { name, .. } => {
                var_declarations.insert(name.clone(), false);
                Ok(())
            }
            Statement::Assign { name, .. } => {
                mutations.insert(name.clone());
                Ok(())
            }
            Statement::For { body, .. } => {
                for stmt in body {
                    self.collect_mutations(stmt, mutations, var_declarations)?;
                }
                Ok(())
            }
            Statement::If { then_block, else_block, .. } => {
                for stmt in then_block {
                    self.collect_mutations(stmt, mutations, var_declarations)?;
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.collect_mutations(stmt, mutations, var_declarations)?;
                    }
                }
                Ok(())
            }
            Statement::While { body, .. } => {
                for stmt in body {
                    self.collect_mutations(stmt, mutations, var_declarations)?;
                }
                Ok(())
            }
            _ => Ok(())
        }
    }
}