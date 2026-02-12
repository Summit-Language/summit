use crate::frontend::ast::*;
use super::program_generator::ProgramGenerator;
use super::statement_generator::StatementGenerator;

/// Generates C code for function declarations and implementations.
pub struct FunctionGenerator<'a> {
    /// The Program Generator responsible for generating the executable program
    generator: &'a mut ProgramGenerator,
}

impl<'a> FunctionGenerator<'a> {
    /// Creates a new FunctionGenerator instance.
    ///
    /// # Parameters
    /// - `generator`: The parent program generator
    pub fn new(generator: &'a mut ProgramGenerator) -> Self {
        FunctionGenerator { generator }
    }

    /// Emits a function declaration.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `func`: The function to declare
    pub fn emit_func_decl(&mut self, func: &Function) {
        self.emit_func_signature(func);
        self.generator.emitter.emit(");\n");
    }

    /// Emits an external function declaration with ABI support.
    ///
    /// External functions are declared with `abi "C"` and no body.
    /// These are functions that exist in C libraries.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `func`: The external function to declare
    ///
    /// # Example
    /// ```summit
    /// abi "C" func printf(fmt: str, args...): i32;
    /// ```
    /// Generates:
    /// ```c
    /// extern int32_t printf(char*, ...);
    /// ```
    pub fn emit_external_func_decl(&mut self, func: &Function) {
        let c_return_type = if func.return_type == "void" {
            "void".to_string()
        } else {
            self.map_summit_to_c_type(&func.return_type)
        };

        self.generator.emitter.emit("extern ");
        self.generator.emitter.emit(&c_return_type);
        self.generator.emitter.emit(" ");
        self.generator.emitter.emit(&func.name);
        self.generator.emitter.emit("(");

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.generator.emitter.emit(", ");
            }
            let c_param_type = self.map_summit_to_c_type(&param.param_type);
            self.generator.emitter.emit(&c_param_type);
        }

        if func.has_varargs {
            if !func.params.is_empty() {
                self.generator.emitter.emit(", ");
            }
            self.generator.emitter.emit("...");
        } else if func.params.is_empty() {
            self.generator.emitter.emit("void");
        }

        self.generator.emitter.emit(");\n");
    }

    /// Maps Summit types to standard C types for external functions.
    /// This is important for C ABI compatibility.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `summit_type`: The Summit type to map
    ///
    /// # Returns
    /// The C type as a string
    ///
    /// # Type Mapping
    /// - `str` → `char*`
    /// - `i8` → `int8_t`
    /// - `i32` → `int32_t`
    /// - `bool` → `int` (C ABI convention)
    /// - etc.
    fn map_summit_to_c_type(&self, summit_type: &str) -> String {
        match summit_type {
            "str" => "char*".to_string(),
            "i8" => "int8_t".to_string(),
            "u8" => "uint8_t".to_string(),
            "i16" => "int16_t".to_string(),
            "u16" => "uint16_t".to_string(),
            "i32" => "int32_t".to_string(),
            "u32" => "uint32_t".to_string(),
            "i64" => "int64_t".to_string(),
            "u64" => "uint64_t".to_string(),
            "i128" => "__int128".to_string(),
            "u128" => "unsigned __int128".to_string(),
            "bool" => "int".to_string(),
            "void" => "void".to_string(),
            _ => {
                if self.generator.struct_defs.contains_key(summit_type) {
                    summit_type.to_string()
                } else {
                    summit_type.to_string()
                }
            }
        }
    }

    /// Generates a complete function implementation.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `func`: The function to generate
    pub fn generate_func(&mut self, func: &Function) {
        if func.body.is_empty() {
            return;
        }

        self.generator.symbol_table.clear();

        for param in &func.params {
            self.generator.symbol_table.insert(param.name.clone(), param.param_type.clone());
        }

        self.emit_func_signature(func);
        self.generator.emitter.emit(") {\n");
        self.generator.emitter.indent_level += 1;

        let mut stmt_gen = StatementGenerator::new(self.generator);
        for stmt in &func.body {
            stmt_gen.generate_stmt(stmt);
        }

        self.generator.emitter.indent_level -= 1;
        self.generator.emitter.emit_line("}");
    }

    /// Emits the function signature (return type, name, and parameters).
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `func`: The function whose signature to emit
    ///
    /// # Note
    /// This does NOT emit the closing parenthesis or semicolon/brace.
    /// The caller is responsible for that.
    fn emit_func_signature(&mut self, func: &Function) {
        let return_type = self.generator.map_type(&func.return_type);
        self.generator.emitter.emit(&format!("{} {}(", return_type, func.name));

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.generator.emitter.emit(", ");
            }
            let param_type = self.generator.map_type(&param.param_type);
            self.generator.emitter.emit(&format!("{} {}", param_type, param.name));
        }

        if func.params.is_empty() && func.abi.is_none() {
            self.generator.emitter.emit("void");
        }
    }
}