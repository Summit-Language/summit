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

    /// Generates a complete function implementation.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `func`: The function to generate
    pub fn generate_func(&mut self, func: &Function) {
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

    /// Emits the function signature.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `func`: The function whose signature to emit
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
    }
}