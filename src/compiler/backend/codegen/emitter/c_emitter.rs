use super::literal_emitter::LiteralEmitter;
use super::operator_emitter::OperatorEmitter;

/// Emits C code with proper formatting and indentation.
pub struct CEmitter {
    /// The accumulated C code output
    pub output: String,

    /// Current indentation level
    pub indent_level: usize,

    /// Handles literal value generation
    literal_emitter: LiteralEmitter,

    /// Handles operator generation
    operator_emitter: OperatorEmitter,
}

impl CEmitter {
    /// Creates a new CEmitter instance.
    pub fn new() -> Self {
        CEmitter {
            output: String::new(),
            indent_level: 0,
            literal_emitter: LiteralEmitter::new(),
            operator_emitter: OperatorEmitter::new(),
        }
    }

    /// Appends raw C code to the output.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `code`: The code to append
    pub fn emit(&mut self, code: &str) {
        self.output.push_str(code);
    }

    /// Appends a line of C code with proper indentation.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `code`: The code line to append
    pub fn emit_line(&mut self, code: &str) {
        self.indent();
        self.output.push_str(code);
        self.output.push('\n');
    }

    /// Adds indentation spaces to the output based on current level.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    pub fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    /// Appends an integer literal to the output.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `value`: The integer value
    pub fn emit_int_literal(&mut self, value: u128) {
        let literal = self.literal_emitter.emit_int_literal(value);
        self.emit(&literal);
    }

    /// Appends a boolean literal to the output.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `value`: The boolean value
    pub fn emit_bool_literal(&mut self, value: bool) {
        let literal = self.literal_emitter.emit_bool_literal(value);
        self.emit(&literal);
    }

    /// Appends a string literal to the output with proper escaping.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `s`: The string content
    pub fn emit_string_literal(&mut self, s: &str) {
        let literal = self.literal_emitter.emit_string_literal(s);
        self.emit(&literal);
    }

    /// Returns the C operator string for a binary operator.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `op`: The binary operator
    ///
    /// # Returns
    /// The C operator as a string
    pub fn emit_binary_op(&self, op: &crate::frontend::ast::BinaryOp) -> String {
        self.operator_emitter.emit_binary_op(op)
    }
}