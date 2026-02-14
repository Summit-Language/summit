#[derive(Debug, Clone)]
pub struct EnumDef {
    /// The enum's name
    pub name: String,

    /// The enum's variants
    pub variants: Vec<EnumVariant>,
}

/// A single variant in an enum
#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// The variant's name
    pub name: String,

    /// The variant's payload types
    pub payload: Option<Vec<String>>,
}