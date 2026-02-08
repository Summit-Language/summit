/// Utility functions for type operations.
pub struct TypeUtils;

impl TypeUtils {
    /// Infers the smallest type that can hold an integer literal.
    ///
    /// # Parameters
    /// - `val`: The integer literal value
    ///
    /// # Returns
    /// The inferred type as a string ("i64", "u64", or "u128")
    pub fn infer_int_literal_type(val: u128) -> String {
        if val <= i64::MAX as u128 {
            "i64".to_string()
        } else if val <= u64::MAX as u128 {
            "u64".to_string()
        } else {
            "u128".to_string()
        }
    }
}