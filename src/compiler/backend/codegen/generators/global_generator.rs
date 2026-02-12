use crate::frontend::ast::*;
use super::program_generator::ProgramGenerator;
use super::expression_generator::ExpressionGenerator;
use crate::utils::type_resolver::TypeResolver;

/// Generates C code for global declarations.
pub struct GlobalGenerator<'a> {
    /// The Program Generator responsible for generating the executable program
    generator: &'a mut ProgramGenerator,
}

impl<'a> GlobalGenerator<'a> {
    /// Creates a new GlobalGenerator instance.
    ///
    /// # Parameters
    /// - `generator`: The parent program generator
    pub fn new(generator: &'a mut ProgramGenerator) -> Self {
        GlobalGenerator { generator }
    }

    /// Emits compile-time global variables with full initialization.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `globals`: Compile-time global declarations
    pub fn emit_comptime_globals(&mut self, globals: &[&GlobalDeclaration]) {
        for global in globals {
            self.emit_global_decl(global);
        }
    }

    /// Emits runtime global variable declarations.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `globals`: Runtime global declarations
    pub fn emit_runtime_global_decls(&mut self, globals: &[&GlobalDeclaration]) {
        for global in globals {
            self.emit_global_decls_only(global);
        }
    }

    /// Emits a complete global declaration with initialization.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `global`: The global declaration
    fn emit_global_decl(&mut self, global: &GlobalDeclaration) {
        match global {
            GlobalDeclaration::Var { name, var_type, 
                value } |
            GlobalDeclaration::Const { name, var_type, 
                value } |
            GlobalDeclaration::Comptime { name, var_type, 
                value } => {
                let summit_type = TypeResolver::resolve_type(
                    var_type, value, |v| self.generator.infer_expr_type(v));
                let c_type = self.generator.map_type(&summit_type).to_string();

                if c_type.starts_with("const ") {
                    self.generator.emitter.emit("static ");
                    self.generator.emitter.emit(&c_type);
                } else {
                    self.generator.emitter.emit("static const ");
                    self.generator.emitter.emit(&c_type);
                }
                self.generator.emitter.emit(" ");
                self.generator.emitter.emit(name);
                self.generator.emitter.emit(" = ");

                let mut expr_gen = ExpressionGenerator::new(self.generator);
                expr_gen.generate_expr(value);

                self.generator.emitter.emit(";\n");

                TypeResolver::register_variable(&mut self.generator.symbol_table, name,
                                                summit_type);
            }
            GlobalDeclaration::Struct(_) => {}
        }
    }

    /// Emits a global declaration without initialization.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `global`: The global declaration
    fn emit_global_decls_only(&mut self, global: &GlobalDeclaration) {
        match global {
            GlobalDeclaration::Var { name, var_type, 
                value } => {
                let summit_type = TypeResolver::resolve_type(
                    var_type, value, |v| self.generator.infer_expr_type(v));
                let c_type = self.generator.map_type(&summit_type).to_string();

                self.generator.emitter.emit("static ");
                self.generator.emitter.emit(&c_type);
                self.generator.emitter.emit(" ");
                self.generator.emitter.emit(name);
                self.generator.emitter.emit(";\n");
            }
            GlobalDeclaration::Const { name, var_type, 
                value } => {
                let summit_type = TypeResolver::resolve_type(
                    var_type, value, |v| self.generator.infer_expr_type(v));
                let c_type = self.generator.map_type(&summit_type).to_string();

                if c_type.starts_with("const ") {
                    self.generator.emitter.emit("static ");
                    self.generator.emitter.emit(&c_type);
                } else {
                    self.generator.emitter.emit("static ");
                    self.generator.emitter.emit(&c_type);
                }
                self.generator.emitter.emit(" ");
                self.generator.emitter.emit(name);
                self.generator.emitter.emit(";\n");
            }
            GlobalDeclaration::Comptime { .. } => {}
            GlobalDeclaration::Struct(_) => {}
        }
    }

    /// Emits initialization code for a runtime global variable.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `global`: The global declaration
    pub fn emit_global_init(&mut self, global: &GlobalDeclaration) {
        match global {
            GlobalDeclaration::Var { name, value, .. } => {
                self.generator.emitter.indent();
                self.generator.emitter.emit(name);
                self.generator.emitter.emit(" = ");

                let mut expr_gen = ExpressionGenerator::new(self.generator);
                expr_gen.generate_expr(value);

                self.generator.emitter.emit(";\n");
            }
            GlobalDeclaration::Const { name, value, .. } => {
                self.generator.emitter.indent();
                self.generator.emitter.emit(name);
                self.generator.emitter.emit(" = ");

                let mut expr_gen = ExpressionGenerator::new(self.generator);
                expr_gen.generate_expr(value);

                self.generator.emitter.emit(";\n");
            }
            GlobalDeclaration::Comptime { .. } => {}
            GlobalDeclaration::Struct(_) => {}
        }
    }
}