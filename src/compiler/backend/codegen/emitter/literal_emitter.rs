/// Generates C literal equivalents for Summit literals.
pub struct LiteralEmitter;

impl LiteralEmitter {
    /// Creates a new LiteralEmitter instance.
    pub fn new() -> Self {
        LiteralEmitter
    }

    /// Generates C code for an integer literal.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `value`: The integer value
    ///
    /// # Returns
    /// The C representation of the integer literal
    pub fn emit_int_literal(&self, value: u128) -> String {
        if value <= i64::MAX as u128 {
            format!("{}", value as i64)
        } else if value <= u64::MAX as u128 {
            format!("{}ULL", value)
        } else {
            if value <= i128::MAX as u128 {
                let high = (value >> 64) as i64;
                let low = value as u64;
                format!("((__int128){}LL << 64 | {}ULL)", high, low)
            } else {
                let high = (value >> 64) as u64;
                let low = value as u64;
                format!("((unsigned __int128){}ULL << 64 | {}ULL)", high, low)
            }
        }
    }

    /// Generates C code for a boolean literal.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `value`: The boolean value
    ///
    /// # Returns
    /// "1" for true, "0" for false
    pub fn emit_bool_literal(&self, value: bool) -> String {
        if value {
            "1".to_string()
        } else {
            "0".to_string()
        }
    }

    /// Generates C code for a string literal.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `s`: The string content
    ///
    /// # Returns
    /// A C string literal with escaped special characters
    pub fn emit_string_literal(&self, s: &str) -> String {
        let mut result = String::from("\"");
        for ch in s.chars() {
            match ch {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                _ => result.push(ch),
            }
        }
        result.push('"');
        result
    }
}