use crate::frontend::ast::*;
use super::type_checker_utils::TypeCheckerUtils;

/// Validates that values fit within their target type bounds.
pub struct BoundsChecker;

impl BoundsChecker {
    /// Creates a new BoundsChecker instance.
    pub fn new() -> Self {
        BoundsChecker
    }

    /// Checks if a literal value fits within the bounds of a target type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `value`: The literal value to check
    /// - `target_type`: The type to validate against
    ///
    /// # Returns
    /// Ok if the value fits, Err with message if out of bounds
    pub fn check_literal_bounds(&self, value: u128, target_type: &str) -> Result<(), String> {
        match target_type {
            "bool" => {
                if value > 1 {
                    return Err(format!(
                        "Integer literal {} cannot be converted to 'bool' (valid values: 0 or 1)",
                        value
                    ));
                }
            }
            "i8" => {
                if value > i8::MAX as u128 {
                    return Err(format!(
                        "Integer literal {} exceeds maximum value for type 'i8' (maximum: {})",
                        value, i8::MAX
                    ));
                }
            }
            "i16" => {
                if value > i16::MAX as u128 {
                    return Err(format!(
                        "Integer literal {} exceeds maximum value for type 'i16' (maximum: {})",
                        value, i16::MAX
                    ));
                }
            }
            "i32" => {
                if value > i32::MAX as u128 {
                    return Err(format!(
                        "Integer literal {} exceeds maximum value for type 'i32' (maximum: {})",
                        value, i32::MAX
                    ));
                }
            }
            "i64" => {
                if value > i64::MAX as u128 {
                    return Err(format!(
                        "Integer literal {} exceeds maximum value for type 'i64' (maximum: {})",
                        value, i64::MAX
                    ));
                }
            }
            "i128" => {
                if value > i128::MAX as u128 {
                    return Err(format!(
                        "Integer literal {} exceeds maximum value for type 'i128' (maximum: {})",
                        value, i128::MAX
                    ));
                }
            }
            "u8" => {
                if value > u8::MAX as u128 {
                    return Err(format!(
                        "Integer literal {} exceeds maximum value for type 'u8' (maximum: {})",
                        value, u8::MAX
                    ));
                }
            }
            "u16" => {
                if value > u16::MAX as u128 {
                    return Err(format!(
                        "Integer literal {} exceeds maximum value for type 'u16' (maximum: {})",
                        value, u16::MAX
                    ));
                }
            }
            "u32" => {
                if value > u32::MAX as u128 {
                    return Err(format!(
                        "Integer literal {} exceeds maximum value for type 'u32' (maximum: {})",
                        value, u32::MAX
                    ));
                }
            }
            "u64" => {
                if value > u64::MAX as u128 {
                    return Err(format!(
                        "Integer literal {} exceeds maximum value for type 'u64' (maximum: {})",
                        value, u64::MAX
                    ));
                }
            }
            "u128" => {
                // u128 can hold any valid literal
            }
            _ => {}
        }

        Ok(())
    }

    /// Checks if an expression's value fits within the bounds of a target type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to check
    /// - `target_type`: The type to validate against
    /// - `_type_utils`: Type utility functions (unused)
    ///
    /// # Returns
    /// Ok if the expression fits, Err with message if out of bounds
    pub fn check_expression_bounds(&self, expr: &Expression, target_type: &str, _type_utils: &TypeCheckerUtils) -> Result<(), String> {
        match expr {
            Expression::IntLiteral(val) => {
                self.check_literal_bounds(*val, target_type)
            }
            Expression::BoolLiteral(_) => {
                if target_type != "bool" {
                    return Err(format!(
                        "Cannot assign boolean literal to type '{}'",
                        target_type
                    ));
                }
                Ok(())
            }
            Expression::Unary { op: UnaryOp::Negate, operand } => {
                self.check_negative_literal_bounds(operand, target_type)
            }
            _ => Ok(())
        }
    }

    /// Checks if a negated literal fits within the bounds of a target type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `operand`: The operand being negated
    /// - `target_type`: The type to validate against
    ///
    /// # Returns
    /// Ok if the negated value fits, Err with message if out of bounds
    fn check_negative_literal_bounds(&self, operand: &Expression, target_type: &str) -> Result<(), String> {
        if let Expression::IntLiteral(val) = operand {
            match target_type {
                "i8" => {
                    if *val > 128 {
                        return Err(format!(
                            "Value -{} is out of bounds for type 'i8' (valid range: {} to {})",
                            val, i8::MIN, i8::MAX
                        ));
                    }
                }
                "i16" => {
                    if *val > 32768 {
                        return Err(format!(
                            "Value -{} is out of bounds for type 'i16' (valid range: {} to {})",
                            val, i16::MIN, i16::MAX
                        ));
                    }
                }
                "i32" => {
                    if *val > 2147483648 {
                        return Err(format!(
                            "Value -{} is out of bounds for type 'i32' (valid range: {} to {})",
                            val, i32::MIN, i32::MAX
                        ));
                    }
                }
                "i64" => {
                    if *val > 9223372036854775808 {
                        return Err(format!(
                            "Value -{} is out of bounds for type 'i64' (valid range: {} to {})",
                            val, i64::MIN, i64::MAX
                        ));
                    }
                }
                "i128" => {
                    if *val > 170141183460469231731687303715884105728 {
                        return Err(format!(
                            "Value -{} is out of bounds for type 'i128' (valid range: -170141183460469231731687303715884105728 to 170141183460469231731687303715884105727)",
                            val
                        ));
                    }
                }
                "u8" | "u16" | "u32" | "u64" | "u128" => {
                    return Err(format!(
                        "Cannot assign negative value -{} to unsigned type '{}'",
                        val, target_type
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }
}