use crate::frontend::ast::BinaryOp;

/// Maps Summit binary operators to their C equivalents.
pub struct OperatorEmitter;

impl OperatorEmitter {
    /// Creates a new OperatorEmitter instance.
    pub fn new() -> Self {
        OperatorEmitter
    }

    /// Returns the C operator string for a Summit binary operator.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `op`: The Summit binary operator
    ///
    /// # Returns
    /// The corresponding C operator as a string
    pub fn emit_binary_op(&self, op: &BinaryOp) -> String {
        match op {
            BinaryOp::Add => "+".to_string(),
            BinaryOp::Sub => "-".to_string(),
            BinaryOp::Mul => "*".to_string(),
            BinaryOp::Div => "/".to_string(),
            BinaryOp::Mod => "%".to_string(),
            BinaryOp::Equal => "==".to_string(),
            BinaryOp::NotEqual => "!=".to_string(),
            BinaryOp::Less => "<".to_string(),
            BinaryOp::Greater => ">".to_string(),
            BinaryOp::LessEqual => "<=".to_string(),
            BinaryOp::GreaterEqual => ">=".to_string(),
            BinaryOp::And => "&&".to_string(),
            BinaryOp::Or => "||".to_string(),
        }
    }
}