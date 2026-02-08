use crate::frontend::ast::*;
use super::type_conversion::TypeConversion;
use super::type_checker_utils::TypeCheckerUtils;
use super::bounds_checker::BoundsChecker;

/// Checks type compatibility and conversion validity.
pub struct TypeCompatibility;

impl TypeCompatibility {
    /// Creates a new TypeCompatibility instance.
    pub fn new() -> Self {
        TypeCompatibility
    }

    /// Checks if an actual type is compatible with an expected type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expected`: The expected type
    /// - `actual`: The actual type
    /// - `expr`: The expression being type-checked
    /// - `type_conversion`: Type conversion utility
    /// - `type_utils`: Type utility functions
    /// - `_bounds_checker`: Bounds checking utility
    ///
    /// # Returns
    /// Ok if types are compatible, Err with message if incompatible
    pub fn check_type_compatibility(&self, expected: &str, actual: &str, expr: &Expression,
                                    type_conversion: &TypeConversion, type_utils: &TypeCheckerUtils,
                                    _bounds_checker: &BoundsChecker) -> Result<(), String> {
        if !self.types_compatible(expected, actual) {
            if type_conversion.can_convert(actual, expected) {
                if type_utils.is_signed(actual) && !type_utils.is_signed(expected) {
                    if let Some(val) = type_utils.get_literal_value(expr) {
                        if val > type_utils.get_unsigned_max(expected) {
                            return Err(format!(
                                "Cannot convert signed value {} to unsigned type '{}' (exceeds maximum: {})",
                                val, expected, type_utils.get_unsigned_max(expected)
                            ));
                        }
                    } else {
                        return Err(format!(
                            "Cannot implicitly convert signed type '{}' to unsigned type '{}' without explicit cast",
                            actual, expected
                        ));
                    }
                }
                Ok(())
            } else {
                Err(format!(
                    "Type mismatch: expected '{}', got '{}'",
                    expected, actual
                ))
            }
        } else {
            Ok(())
        }
    }

    /// Checks if two types are compatible.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type1`: The first type
    /// - `type2`: The second type
    ///
    /// # Returns
    /// True if the types are compatible
    pub fn types_compatible(&self, type1: &str, type2: &str) -> bool {
        if type1 == type2 {
            return true;
        }

        let int_types = ["i8", "i16", "i32", "i64", "i128",
            "u8", "u16", "u32", "u64", "u128"];
        let both_ints = int_types.contains(&type1) && int_types.contains(&type2);

        both_ints
    }
}