use super::expressions::Expression;

/// A struct definition at the global level.
#[derive(Debug, Clone)]
pub struct StructDef {
    /// The struct's name
    pub name: String,

    /// The struct's fields
    pub fields: Vec<StructField>,
}

/// A field in a struct definition.
#[derive(Debug, Clone)]
pub struct StructField {
    /// The field's name
    pub name: String,

    /// The field's type
    pub field_type: String,
}

/// A field initializer in a struct instantiation.
#[derive(Debug, Clone)]
pub struct StructFieldInit {
    /// The field name
    pub name: Option<String>,

    /// The initialization value
    pub value: Expression,
}