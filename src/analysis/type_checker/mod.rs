mod type_inference;
mod type_compatibility;
mod type_conversion;
mod bounds_checker;
mod type_checker_utils;

pub use type_inference::TypeInference;
pub use type_compatibility::TypeCompatibility;
pub use type_conversion::TypeConversion;
pub use bounds_checker::BoundsChecker;
pub use type_checker_utils::TypeCheckerUtils;

use crate::frontend::ast::*;
use std::collections::HashMap;

/// Main type checker that does type analysis.
pub struct TypeChecker {
    type_inference: TypeInference,
    type_compatibility: TypeCompatibility,
    type_conversion: TypeConversion,
    bounds_checker: BoundsChecker,
    type_utils: TypeCheckerUtils,
}

impl TypeChecker {
    /// Creates a new type checker.
    ///
    /// # Returns
    /// A new TypeChecker instance with all subcomponents initialized
    pub fn new() -> Self {
        TypeChecker {
            type_inference: TypeInference::new(),
            type_compatibility: TypeCompatibility::new(),
            type_conversion: TypeConversion::new(),
            bounds_checker: BoundsChecker::new(),
            type_utils: TypeCheckerUtils::new(),
        }
    }

    /// Infers the type of expression.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to analyze
    /// - `scope`: Current variable scope mapping names to types
    /// - `functions`: Available function definitions
    ///
    /// # Returns
    /// The inferred type name, or Err with message on failure
    pub fn infer_type(&self, expr: &Expression, scope: &HashMap<String, String>,
                      functions: &HashMap<String, Function>) -> Result<String, String> {
        self.type_inference.infer_type(expr, scope, functions, &self.type_utils)
    }

    /// Checks if an actual type is compatible with an expected type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expected`: The expected type
    /// - `actual`: The actual type
    /// - `expr`: The expression being checked
    ///
    /// # Returns
    /// Ok(()) if types are compatible, Err with message on failure
    pub fn check_type_compatibility(&self, expected: &str, actual: &str,
                                    expr: &Expression) -> Result<(), String> {
        self.type_compatibility.check_type_compatibility(expected, actual, expr,
                                                         &self.type_conversion, &self.type_utils,
                                                         &self.bounds_checker)
    }

    /// Checks if two types are compatible.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type1`: First type to compare
    /// - `type2`: Second type to compare
    ///
    /// # Returns
    /// True if the types are compatible
    pub fn types_compatible(&self, type1: &str, type2: &str) -> bool {
        self.type_compatibility.types_compatible(type1, type2)
    }

    /// Checks if a type can be implicitly converted to another.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `from`: Source type
    /// - `to`: Target type
    ///
    /// # Returns
    /// True if conversion is allowed
    pub fn can_convert(&self, from: &str, to: &str) -> bool {
        self.type_conversion.can_convert(from, to)
    }

    /// Checks if converting from one type to another would lose data.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `from`: Source type
    /// - `to`: Target type
    ///
    /// # Returns
    /// True if conversion would truncate values
    pub fn would_truncate(&self, from: &str, to: &str) -> bool {
        self.type_conversion.would_truncate(from, to)
    }

    /// Determines the wider type between two types.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type1`: First type
    /// - `type2`: Second type
    ///
    /// # Returns
    /// The wider type that can hold values from both types
    pub fn wider_type(&self, type1: &str, type2: &str) -> String {
        self.type_utils.wider_type(type1, type2)
    }

    /// Checks if a type is a signed integer type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The type to check
    ///
    /// # Returns
    /// True if the type is signed
    pub fn is_signed(&self, type_name: &str) -> bool {
        self.type_utils.is_signed(type_name)
    }

    /// Gets the size in bits of a type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The type to query
    ///
    /// # Returns
    /// Size in bits
    pub fn get_type_size(&self, type_name: &str) -> usize {
        self.type_utils.get_type_size(type_name)
    }

    /// Extracts a literal integer value from an expression.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to check
    ///
    /// # Returns
    /// Some(value) if expression is an integer literal, None otherwise
    pub fn get_literal_value(&self, expr: &Expression) -> Option<u128> {
        self.type_utils.get_literal_value(expr)
    }

    /// Gets the maximum value for a type as a string.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The type to query
    ///
    /// # Returns
    /// String representation of the maximum value
    pub fn get_type_max(&self, type_name: &str) -> String {
        self.type_utils.get_type_max(type_name)
    }

    /// Gets the maximum value for an unsigned type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The unsigned type to query
    ///
    /// # Returns
    /// Maximum value as u128
    pub fn get_unsigned_max(&self, type_name: &str) -> u128 {
        self.type_utils.get_unsigned_max(type_name)
    }

    /// Checks if an expression's value fits within a target type's bounds.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to check
    /// - `target_type`: The type to validate against
    ///
    /// # Returns
    /// Ok(()) if bounds are satisfied, Err with message on failure
    pub fn check_expression_bounds(&self, expr: &Expression,
                                   target_type: &str) -> Result<(), String> {
        self.bounds_checker.check_expression_bounds(expr, target_type, &self.type_utils)
    }
}