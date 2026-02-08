/// Maps Summit types to their equivalent C types.
pub struct TypeMapper;

impl TypeMapper {
    /// Creates a new TypeMapper instance.
    pub fn new() -> Self {
        TypeMapper
    }

    /// Converts a Summit type name to its corresponding C type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The Summit type name to convert
    ///
    /// # Returns
    /// The equivalent C type as a string slice
    pub fn map_type(&self, type_name: &str) -> &str {
        match type_name {
            "bool" => "bool",
            "i8" => "int8_t",
            "i16" => "int16_t",
            "i32" => "int32_t",
            "i64" => "int64_t",
            "i128" => "__int128",
            "u8" => "uint8_t",
            "u16" => "uint16_t",
            "u32" => "uint32_t",
            "u64" => "uint64_t",
            "u128" => "unsigned __int128",
            "void" => "void",
            "str" => "const char*",
            "void*" => "void*",
            _ => "int64_t",
        }
    }
}