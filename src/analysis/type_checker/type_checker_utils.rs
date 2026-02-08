use crate::frontend::ast::*;

/// Utility functions for type operations and analysis.
pub struct TypeCheckerUtils;

impl TypeCheckerUtils {
    /// Creates a new TypeCheckerUtils instance.
    pub fn new() -> Self {
        TypeCheckerUtils
    }

    /// Returns the wider of two numeric types.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type1`: The first type
    /// - `type2`: The second type
    ///
    /// # Returns
    /// The wider type as a string
    pub fn wider_type(&self, type1: &str, type2: &str) -> String {
        let size1 = self.get_type_size(type1);
        let size2 = self.get_type_size(type2);

        if size1 > size2 {
            type1.to_string()
        } else if size2 > size1 {
            type2.to_string()
        } else {
            if self.is_signed(type1) {
                type1.to_string()
            } else {
                type2.to_string()
            }
        }
    }

    /// Checks if a type is a signed integer type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The type name to check
    ///
    /// # Returns
    /// True if the type is signed
    pub fn is_signed(&self, type_name: &str) -> bool {
        type_name.starts_with('i')
    }

    /// Returns the bit size of a type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The type name
    ///
    /// # Returns
    /// The size in bits
    pub fn get_type_size(&self, type_name: &str) -> usize {
        match type_name {
            "bool" => 8,
            "i8" | "u8" => 8,
            "i16" | "u16" => 16,
            "i32" | "u32" => 32,
            "i64" | "u64" => 64,
            "i128" | "u128" => 128,
            _ => 64,
        }
    }

    /// Extracts the literal value from an integer literal expression.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to analyze
    ///
    /// # Returns
    /// Some(value) if the expression is an integer literal
    pub fn get_literal_value(&self, expr: &Expression) -> Option<u128> {
        match expr {
            Expression::IntLiteral(val) => Some(*val),
            _ => None,
        }
    }

    /// Returns the maximum value for a given type as a string.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The type name
    ///
    /// # Returns
    /// The maximum value as a string
    pub fn get_type_max(&self, type_name: &str) -> String {
        match type_name {
            "bool" => "1".to_string(),
            "i8" => i8::MAX.to_string(),
            "i16" => i16::MAX.to_string(),
            "i32" => i32::MAX.to_string(),
            "i64" => i64::MAX.to_string(),
            "i128" => i128::MAX.to_string(),
            "u8" => u8::MAX.to_string(),
            "u16" => u16::MAX.to_string(),
            "u32" => u32::MAX.to_string(),
            "u64" => u64::MAX.to_string(),
            "u128" => u128::MAX.to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Returns the maximum value for an unsigned type as u128.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The unsigned type name
    ///
    /// # Returns
    /// The maximum value as u128
    pub fn get_unsigned_max(&self, type_name: &str) -> u128 {
        match type_name {
            "bool" => 1,
            "u8" => u8::MAX as u128,
            "u16" => u16::MAX as u128,
            "u32" => u32::MAX as u128,
            "u64" => u64::MAX as u128,
            "u128" => u128::MAX,
            _ => 0,
        }
    }
}