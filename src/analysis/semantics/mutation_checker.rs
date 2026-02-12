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
            Statement::Var { name, .. } => {
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
            Statement::FieldAssign { object, .. } => {
                // Track field mutations as mutations of the parent object
                mutations.insert(object.clone());
                Ok(())
            }
            Statement::For { body, .. } => {
                for stmt in body {
                    self.collect_mutations(stmt, mutations, var_declarations)?;
                }
                Ok(())
            }
            Statement::If { then_block, elseif_blocks, else_block, .. } => {
                for stmt in then_block {
                    self.collect_mutations(stmt, mutations, var_declarations)?;
                }
                for elseif in elseif_blocks {
                    for stmt in &elseif.body {
                        self.collect_mutations(stmt, mutations, var_declarations)?;
                    }
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
            Statement::When { cases, else_block, .. } => {
                for case in cases {
                    for stmt in &case.body {
                        self.collect_mutations(stmt, mutations, var_declarations)?;
                    }
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.collect_mutations(stmt, mutations, var_declarations)?;
                    }
                }
                Ok(())
            }
            Statement::Expect { else_block, .. } => {
                for stmt in else_block {
                    self.collect_mutations(stmt, mutations, var_declarations)?;
                }
                Ok(())
            }
            _ => Ok(())
        }
    }

    /// Validates that mutations are only performed on mutable variables.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `mutations`: Set of variable names that are mutated
    /// - `var_declarations`: Map of variable names to mutability (true = mutable)
    ///
    /// # Returns
    /// Ok(()) if all mutations are valid, Err with message on failure
    pub fn validate_mutations(&self, mutations: &HashSet<String>,
                              var_declarations: &HashMap<String, bool>) -> Result<(), String> {
        for var_name in mutations {
            if let Some(&is_mutable) = var_declarations.get(var_name) {
                if !is_mutable {
                    return Err(format!(
                        "Cannot assign to immutable variable '{}'. Use 'var' instead of 'const' or 'comptime' to make it mutable",
                        var_name
                    ));
                }
            }
        }
        Ok(())
    }
}