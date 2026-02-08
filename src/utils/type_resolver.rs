use crate::frontend::ast::*;
use std::collections::HashMap;

/// Utility for resolving and registering variable types.
pub struct TypeResolver;

impl TypeResolver {
    /// Resolves the Summit type for a variable declaration.
    ///
    /// Uses the explicit type annotation if provided, otherwise infers
    /// the type from the initialization expression.
    ///
    /// # Parameters
    /// - `var_type`: Optional type annotation
    /// - `value`: Initialization expression
    /// - `infer_fn`: Function to infer type from expression
    ///
    /// # Returns
    /// The Summit type as a string
    pub fn resolve_type<F>(var_type: &Option<String>, value: &Expression, infer_fn: F) -> String
    where
        F: FnOnce(&Expression) -> String,
    {
        if let Some(t) = var_type {
            t.clone()
        } else {
            infer_fn(value)
        }
    }

    /// Registers a variable in the symbol table using an already-resolved type.
    ///
    /// # Parameters
    /// - `symbol_table`: The symbol table to register in
    /// - `name`: Variable name
    /// - `summit_type`: The resolved Summit type
    pub fn register_variable(
        symbol_table: &mut HashMap<String, String>,
        name: &str,
        summit_type: String,
    ) {
        symbol_table.insert(name.to_string(), summit_type);
    }
}