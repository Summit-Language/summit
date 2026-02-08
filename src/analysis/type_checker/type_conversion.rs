/// Handles type conversion and compatibility checking.
pub struct TypeConversion;

impl TypeConversion {
    /// Creates a new TypeConversion instance.
    pub fn new() -> Self {
        TypeConversion
    }

    /// Checks if a type can be safely converted to another type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `from`: The source type
    /// - `to`: The target type
    ///
    /// # Returns
    /// True if conversion is allowed without data loss
    pub fn can_convert(&self, from: &str, to: &str) -> bool {
        let conversions = [
            ("i8", "i16"), ("i8", "i32"), ("i8", "i64"), ("i8", "i128"),
            ("i16", "i32"), ("i16", "i64"), ("i16", "i128"),
            ("i32", "i64"), ("i32", "i128"),
            ("i64", "i128"),
            ("u8", "u16"), ("u8", "u32"), ("u8", "u64"), ("u8", "u128"),
            ("u16", "u32"), ("u16", "u64"), ("u16", "u128"),
            ("u32", "u64"), ("u32", "u128"),
            ("u64", "u128"),
        ];

        conversions.iter().any(|(f, t)| f == &from && t == &to)
    }

    /// Checks if converting from one type to another would truncate data.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `from`: The source type
    /// - `to`: The target type
    ///
    /// # Returns
    /// True if the conversion would lose precision or range
    pub fn would_truncate(&self, from: &str, to: &str) -> bool {
        let type_sizes = [
            ("i8", 8), ("i16", 16), ("i32", 32), ("i64", 64), ("i128", 128),
            ("u8", 8), ("u16", 16), ("u32", 32), ("u64", 64), ("u128", 128),
        ];

        let from_size = type_sizes.iter()
            .find(|(t, _)| t == &from).map(|(_, s)| s);
        let to_size = type_sizes.iter()
            .find(|(t, _)| t == &to).map(|(_, s)| s);

        match (from_size, to_size) {
            (Some(fs), Some(ts)) => fs > ts,
            _ => false,
        }
    }
}