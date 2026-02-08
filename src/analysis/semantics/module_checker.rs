use super::analyzer::SemanticAnalyzer;

/// Validates module imports and type constraints.
///
/// Ensures that functions called from modules are properly imported
/// and checks type validity for specific operations like I/O.
pub struct ModuleChecker<'a> {
    /// The main program analyzer checking for semantic correctness
    analyzer: &'a SemanticAnalyzer,
}

impl<'a> ModuleChecker<'a> {
    /// Creates a new module checker.
    ///
    /// # Parameters
    /// - `analyzer`: Reference to the semantic analyzer
    ///
    /// # Returns
    /// A new ModuleChecker instance
    pub fn new(analyzer: &'a SemanticAnalyzer) -> Self {
        ModuleChecker { analyzer }
    }

    /// Checks if a module path has been imported.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `path`: The module path components to check
    ///
    /// # Returns
    /// Ok(()) if module is imported, Err with message if not found
    pub fn check_module_import(&self, path: &[String]) -> Result<(), String> {
        let mut found = false;

        if path.len() == 2 {
            let full_path = vec!["std".to_string(), path[0].clone()];
            if self.analyzer.imported_modules.contains(&full_path) {
                found = true;
            }
            if self.analyzer.imported_modules.contains(&vec![path[0].clone()]) {
                found = true;
            }
        } else if path.len() == 3 {
            let parent = vec![path[0].clone()];
            let full = vec![path[0].clone(), path[1].clone()];
            if self.analyzer.imported_modules.contains(&parent)
                || self.analyzer.imported_modules.contains(&full) {
                found = true;
            }
        }

        for i in 1..path.len() {
            if self.analyzer.imported_modules.contains(&path[..i].to_vec()) {
                found = true;
                break;
            }
        }

        if !found {
            let module_name = if path.len() == 2 {
                path[0].clone()
            } else {
                path[..path.len()-1].join("::")
            };
            return Err(format!("Module '{}' not imported", module_name));
        }

        Ok(())
    }

    /// Checks if a type is valid for I/O read operations.
    ///
    /// # Parameters
    /// - `type_name`: The type name to validate
    ///
    /// # Returns
    /// True if the type can be used with io::read
    pub fn is_valid_read_type(&self, type_name: &str) -> bool {
        matches!(type_name, "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64")
    }

    /// Checks if a type is an integer type.
    ///
    /// # Parameters
    /// - `type_name`: The type name to check
    ///
    /// # Returns
    /// True if the type is any signed or unsigned integer type
    pub fn is_integer_type(&self, type_name: &str) -> bool {
        matches!(type_name, "i8" | "i16" | "i32" | "i64" | "i128"
            | "u8" | "u16" | "u32" | "u64" | "u128")
    }
}