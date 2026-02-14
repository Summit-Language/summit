use crate::frontend::ast::*;
use super::super::emitter::CEmitter;
use super::super::helpers::{TypeMapper, TypeInference, CompileTimeChecker};
use super::super::stdlib::{StdlibCollector, StdlibEmitter};
use super::global_generator::GlobalGenerator;
use super::function_generator::FunctionGenerator;
use super::statement_generator::StatementGenerator;
use std::collections::{HashMap, HashSet};

/// Generates C code for an entire Summit program.
pub struct ProgramGenerator {
    /// C code emitter
    pub emitter: CEmitter,

    /// Maps variable names to their types
    pub symbol_table: HashMap<String, String>,

    /// Maps variable names to their mutability
    pub mutability_table: HashMap<String, bool>,

    /// Maps function names to their return types
    pub function_signatures: HashMap<String, String>,

    /// Maps struct names to their definitions
    pub struct_defs: HashMap<String, StructDef>,

    /// Maps enum names to their definitions
    pub enum_defs: HashMap<String, EnumDef>,

    /// Tracks which standard library functions are used
    pub used_stdlib_functions: HashSet<String>,

    /// Maps Summit types to C types
    type_mapper: TypeMapper,

    /// Checks if expressions can be evaluated at compile time
    compile_time_checker: CompileTimeChecker,

    /// Emits standard library function declarations
    stdlib_emitter: StdlibEmitter,

    /// Libraries to link against
    link_libs: Vec<String>,
}

impl ProgramGenerator {
    /// Creates a new ProgramGenerator instance.
    pub fn new(link_libs: Vec<String>) -> Self {
        ProgramGenerator {
            emitter: CEmitter::new(),
            symbol_table: HashMap::new(),
            mutability_table: HashMap::new(),
            function_signatures: HashMap::new(),
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            used_stdlib_functions: HashSet::new(),
            type_mapper: TypeMapper::new(),
            compile_time_checker: CompileTimeChecker::new(),
            stdlib_emitter: StdlibEmitter::new(),
            link_libs,
        }
    }

    /// Checks if we're linking against libc
    fn linking_libc(&self) -> bool {
        self.link_libs.contains(&"c".to_string())
    }

    /// Generates C code for a complete Summit program.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `program`: The Summit program AST to generate code for
    ///
    /// # Returns
    /// The generated C code as a string
    pub fn generate_program(&mut self, program: &Program) -> String {
        let mut structs = Vec::new();
        let mut enums = Vec::new();

        for global in &program.globals {
            match global {
                GlobalDeclaration::Struct(struct_def) => {
                    self.struct_defs.insert(struct_def.name.clone(), struct_def.clone());
                    structs.push(struct_def.clone());
                }
                GlobalDeclaration::Enum(enum_def) => {
                    self.enum_defs.insert(enum_def.name.clone(), enum_def.clone());
                    enums.push(enum_def.clone());
                }
                _ => {}
            }
        }

        for func in &program.functions {
            self.function_signatures.insert(func.name.clone(), func.return_type.clone());
        }

        let runtime_global_names = self.identify_runtime_globals(&program.globals);

        let (comptime_globals, runtime_globals)
            = self.separate_globals(&program.globals, &runtime_global_names);

        self.collect_stdlib_usage(program);
        self.emit_headers();

        if !self.used_stdlib_functions.is_empty() {
            self.emit_stdlib_decls();
        }

        self.emit_enum_defs(&enums);
        self.emit_struct_defs(&structs);

        let mut global_gen = GlobalGenerator::new(self);
        global_gen.emit_comptime_globals(&comptime_globals);
        global_gen.emit_runtime_global_decls(&runtime_globals);

        if !program.globals.is_empty() {
            self.emitter.emit_line("");
        }

        let (external_funcs, regular_funcs): (Vec<_>, Vec<_>) = program.functions
            .iter()
            .partition(|f| f.body.is_empty() && f.abi.is_some());

        for func in &external_funcs {
            let mut func_gen = FunctionGenerator::new(self);
            func_gen.emit_external_func_decl(func);
        }

        if !external_funcs.is_empty() {
            self.emitter.emit_line("");
        }

        for func in &regular_funcs {
            let mut func_gen = FunctionGenerator::new(self);
            func_gen.emit_func_decl(func);
        }

        if !regular_funcs.is_empty() {
            self.emitter.emit_line("");
        }

        self.generate_main_or_funcs(program, &runtime_globals, &regular_funcs);

        self.emitter.output.clone()
    }

    /// Emits C struct definitions.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `structs`: The struct definitions to emit
    fn emit_struct_defs(&mut self, structs: &[StructDef]) {
        if structs.is_empty() {
            return;
        }

        for struct_def in structs {
            self.emitter.emit(&format!("typedef struct {} {{\n", struct_def.name));
            self.emitter.indent_level += 1;

            for field in &struct_def.fields {
                self.emitter.indent();
                let c_type = self.map_type(&field.field_type);
                self.emitter.emit(&format!("{} {};\n", c_type, field.name));
            }

            self.emitter.indent_level -= 1;
            self.emitter.emit(&format!("}} {};\n", struct_def.name));
            self.emitter.emit_line("");
        }
    }

    /// Emits C enum definitions as tagged unions.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `enums`: The enum definitions to emit
    fn emit_enum_defs(&mut self, enums: &[EnumDef]) {
        if enums.is_empty() {
            return;
        }

        for enum_def in enums {
            // Emit tag enum
            self.emitter.emit(&format!("typedef enum {{\n"));
            self.emitter.indent_level += 1;

            for variant in &enum_def.variants {
                self.emitter.indent();
                self.emitter.emit(&format!("{}_{},\n", enum_def.name, variant.name));
            }

            self.emitter.indent_level -= 1;
            self.emitter.emit(&format!("}} {}_Tag;\n\n", enum_def.name));

            // Emit the tagged union
            self.emitter.emit(&format!("typedef struct {{\n"));
            self.emitter.indent_level += 1;

            self.emitter.indent();
            self.emitter.emit(&format!("{}_Tag tag;\n", enum_def.name));

            // If any variants have payloads, emit a union
            let has_payloads = enum_def.variants.iter().any(|v| v.payload.is_some());
            if has_payloads {
                self.emitter.indent();
                self.emitter.emit("union {\n");
                self.emitter.indent_level += 1;

                for variant in &enum_def.variants {
                    if let Some(payload_types) = &variant.payload {
                        self.emitter.indent();
                        if payload_types.len() == 1 {
                            let c_type = self.map_type(&payload_types[0]);
                            self.emitter.emit(&format!("{} {};\n", c_type, variant.name.to_lowercase()));
                        } else {
                            self.emitter.emit(&format!("struct {{\n"));
                            self.emitter.indent_level += 1;
                            for (i, payload_type) in payload_types.iter().enumerate() {
                                self.emitter.indent();
                                let c_type = self.map_type(payload_type);
                                self.emitter.emit(&format!("{} _{};\n", c_type, i));
                            }
                            self.emitter.indent_level -= 1;
                            self.emitter.indent();
                            self.emitter.emit(&format!("}} {};\n", variant.name.to_lowercase()));
                        }
                    }
                }

                self.emitter.indent_level -= 1;
                self.emitter.indent();
                self.emitter.emit("} data;\n");
            }

            self.emitter.indent_level -= 1;
            self.emitter.emit(&format!("}} {};\n", enum_def.name));
            self.emitter.emit_line("");
        }
    }

    /// Identifies which globals require runtime initialization.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `globals`: The global declarations to analyze
    ///
    /// # Returns
    /// A set containing names of globals that require runtime initialization
    fn identify_runtime_globals(&self, globals: &[GlobalDeclaration]) -> HashSet<String> {
        let mut runtime_global_names = HashSet::new();

        for global in globals {
            match global {
                GlobalDeclaration::Var { name, .. } => {
                    runtime_global_names.insert(name.clone());
                }
                GlobalDeclaration::Const { name, value, .. } => {
                    if !self.compile_time_checker
                        .is_compile_time_constant_expr(value, &runtime_global_names) {
                        runtime_global_names.insert(name.clone());
                    }
                }
                GlobalDeclaration::Comptime { .. } => {}
                GlobalDeclaration::Struct(_) => {}
                GlobalDeclaration::Enum(_) => {}
            }
        }

        runtime_global_names
    }

    /// Separates globals into compile-time and runtime categories.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `globals`: All global declarations
    /// - `runtime_global_names`: Names of globals requiring runtime initialization
    ///
    /// # Returns
    /// A tuple containing compile-time globals and runtime globals
    fn separate_globals<'a>(
        &mut self,
        globals: &'a [GlobalDeclaration],
        runtime_global_names: &HashSet<String>,
    ) -> (Vec<&'a GlobalDeclaration>, Vec<&'a GlobalDeclaration>) {
        let mut comptime_globals = Vec::new();
        let mut runtime_globals = Vec::new();

        for global in globals {
            match global {
                GlobalDeclaration::Var { name, var_type, value } => {
                    let inferred_type = if let Some(t) = var_type {
                        t.clone()
                    } else {
                        self.infer_expr_type(value)
                    };
                    self.symbol_table.insert(name.clone(), inferred_type);
                    self.mutability_table.insert(name.clone(), true);
                    runtime_globals.push(global);
                }
                GlobalDeclaration::Comptime { name, var_type, value } => {
                    let inferred_type = if let Some(t) = var_type {
                        t.clone()
                    } else {
                        self.infer_expr_type(value)
                    };
                    self.symbol_table.insert(name.clone(), inferred_type);
                    self.mutability_table.insert(name.clone(), false);
                    comptime_globals.push(global);
                }
                GlobalDeclaration::Const { name, var_type, value } => {
                    let inferred_type = if let Some(t) = var_type {
                        t.clone()
                    } else {
                        self.infer_expr_type(value)
                    };
                    self.symbol_table.insert(name.clone(), inferred_type);
                    self.mutability_table.insert(name.clone(), false);

                    if runtime_global_names.contains(name) {
                        runtime_globals.push(global);
                    } else {
                        comptime_globals.push(global);
                    }
                }
                GlobalDeclaration::Struct(_) => {}
                GlobalDeclaration::Enum(_) => {}
            }
        }

        (comptime_globals, runtime_globals)
    }

    /// Collects standard library functions used in the program.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `program`: The program to analyze for stdlib usage
    fn collect_stdlib_usage(&mut self, program: &Program) {
        let mut collector = StdlibCollector::new(&mut self.used_stdlib_functions,
                                                 &mut self.symbol_table);
        for func in &program.functions {
            if func.body.is_empty() {
                continue;
            }
            collector.collect_from_func(func);
        }

        for stmt in &program.statements {
            collector.collect_from_stmt(stmt);
        }

        for global in &program.globals {
            match global {
                GlobalDeclaration::Var { value, .. } |
                GlobalDeclaration::Const { value, .. } |
                GlobalDeclaration::Comptime { value, .. } => {
                    collector.collect_from_expr(value);
                }
                GlobalDeclaration::Struct(_) => {}
                GlobalDeclaration::Enum(_) => {}
            }
        }
    }

    /// Emits C header includes.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    fn emit_headers(&mut self) {
        self.emitter.emit_line("#include \"freestanding.h\"");
        self.emitter.emit_line("");
    }

    /// Emits declarations for used standard library functions.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    fn emit_stdlib_decls(&mut self) {
        let func_names: Vec<String> = self.used_stdlib_functions.iter().cloned().collect();
        for func_name in func_names {
            self.stdlib_emitter.emit_decl(&mut self.emitter, &func_name);
        }
        self.emitter.emit_line("");
    }

    /// Generates either a synthetic main function or regular functions.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `program`: The program to generate code for
    /// - `runtime_globals`: Globals requiring runtime initialization
    /// - `regular_funcs`: Functions that are not external declarations
    fn generate_main_or_funcs(&mut self, program: &Program,
                              runtime_globals: &[&GlobalDeclaration],
                              regular_funcs: &[&Function]) {
        let has_main = regular_funcs.iter().any(|f| f.name == "main");

        if !program.statements.is_empty() && !has_main {
            self.emit_synthetic_start(program, runtime_globals);
        } else {
            if has_main && !runtime_globals.is_empty() {
                self.emit_global_init_function(runtime_globals);
            }

            for func in regular_funcs {
                let mut func_gen = FunctionGenerator::new(self);
                func_gen.generate_func(func);
                self.emitter.emit_line("");
            }

            if has_main {
                self.emit_start_wrapper(!runtime_globals.is_empty());
            }
        }
    }

    /// Emits a function to initialize runtime globals.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `runtime_globals`: Globals requiring runtime initialization
    fn emit_global_init_function(&mut self, runtime_globals: &[&GlobalDeclaration]) {
        self.emitter.emit_line("static void __init_globals(void) {");
        self.emitter.indent_level += 1;

        let mut global_gen = GlobalGenerator::new(self);
        for global in runtime_globals {
            global_gen.emit_global_init(global);
        }

        self.emitter.indent_level -= 1;
        self.emitter.emit_line("}");
        self.emitter.emit_line("");
    }

    /// Emits a _start function that calls main and exits.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `has_global_init`: Whether there are runtime globals to initialize
    fn emit_start_wrapper(&mut self, has_global_init: bool) {
        if self.linking_libc() {
            if has_global_init {
                self.emitter.emit_line("// Global initializers will be called before main");
                self.emitter.emit_line("// via constructor attribute or explicit call in main");
                self.emitter.emit_line("");
            }
            return;
        }

        self.emitter.emit_line("void _start(void) {");
        self.emitter.indent_level += 1;

        if has_global_init {
            self.emitter.emit_line("__init_globals();");
        }

        let main_return_type = self.function_signatures.get("main")
            .map(|s| s.as_str())
            .unwrap_or("void");

        if main_return_type == "void" {
            self.emitter.emit_line("main();");
            self.emitter.emit_line("syscall1(SYS_exit, 0);");
        } else {
            self.emitter.emit_line("int8_t exit_code = main();");
            self.emitter.emit_line("syscall1(SYS_exit, exit_code);");
        }

        self.emitter.indent_level -= 1;
        self.emitter.emit_line("}");
        self.emitter.emit_line("");
    }

    /// Emits a synthetic _start function for scripts without an explicit main.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `program`: The program containing top-level statements
    /// - `runtime_globals`: Globals requiring runtime initialization
    fn emit_synthetic_start(&mut self, program: &Program, runtime_globals: &[&GlobalDeclaration]) {
        if self.linking_libc() {
            self.emitter.emit_line("int main(void) {");
            self.emitter.indent_level += 1;

            let mut global_gen = GlobalGenerator::new(self);
            for global in runtime_globals {
                global_gen.emit_global_init(global);
            }

            let mut stmt_gen = StatementGenerator::new(self);
            for stmt in &program.statements {
                stmt_gen.generate_stmt(stmt);
            }

            self.emitter.emit_line("return 0;");
            self.emitter.indent_level -= 1;
            self.emitter.emit_line("}");
            self.emitter.emit_line("");
            return;
        }

        self.emitter.emit_line("void _start(void) {");
        self.emitter.indent_level += 1;

        let mut global_gen = GlobalGenerator::new(self);
        for global in runtime_globals {
            global_gen.emit_global_init(global);
        }

        let mut stmt_gen = StatementGenerator::new(self);
        for stmt in &program.statements {
            stmt_gen.generate_stmt(stmt);
        }

        self.emitter.emit_line("syscall1(SYS_exit, 0);");

        self.emitter.indent_level -= 1;
        self.emitter.emit_line("}");
        self.emitter.emit_line("");
    }

    /// Infers the type of expression.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to analyze
    ///
    /// # Returns
    /// The inferred type as a string
    pub fn infer_expr_type(&self, expr: &Expression) -> String {
        let type_inference = TypeInference::new(&self.symbol_table,
                                                &self.function_signatures,
                                                &self.struct_defs);
        type_inference.infer_expression_type(expr)
    }

    /// Maps a Summit type to its equivalent C type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The Summit type name
    ///
    /// # Returns
    /// The equivalent C type as a string
    pub fn map_type(&self, type_name: &str) -> String {
        if self.struct_defs.contains_key(type_name) {
            return type_name.to_string();
        }
        if self.enum_defs.contains_key(type_name) {
            return type_name.to_string();
        }
        self.type_mapper.map_type(type_name).to_string()
    }
}