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

    /// Maps function names to their return types
    pub function_signatures: HashMap<String, String>,

    /// Tracks which standard library functions are used
    pub used_stdlib_functions: HashSet<String>,

    /// Maps Summit types to C types
    type_mapper: TypeMapper,

    /// Checks if expressions can be evaluated at compile time
    compile_time_checker: CompileTimeChecker,

    /// Emits standard library function declarations
    stdlib_emitter: StdlibEmitter,
}

impl ProgramGenerator {
    /// Creates a new ProgramGenerator instance.
    pub fn new() -> Self {
        ProgramGenerator {
            emitter: CEmitter::new(),
            symbol_table: HashMap::new(),
            function_signatures: HashMap::new(),
            used_stdlib_functions: HashSet::new(),
            type_mapper: TypeMapper::new(),
            compile_time_checker: CompileTimeChecker::new(),
            stdlib_emitter: StdlibEmitter::new(),
        }
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
        for func in &program.functions {
            self.function_signatures.insert(func.name.clone(), func.return_type.clone());
        }

        let runtime_global_names = self.identify_runtime_globals(&program.globals);

        let (comptime_globals, runtime_globals) = self.separate_globals(&program.globals, &runtime_global_names);

        self.collect_stdlib_usage(program);
        self.emit_headers();

        if !self.used_stdlib_functions.is_empty() {
            self.emit_stdlib_decls();
        }

        let mut global_gen = GlobalGenerator::new(self);
        global_gen.emit_comptime_globals(&comptime_globals);
        global_gen.emit_runtime_global_decls(&runtime_globals);

        if !program.globals.is_empty() {
            self.emitter.emit_line("");
        }

        for func in &program.functions {
            let mut func_gen = FunctionGenerator::new(self);
            func_gen.emit_func_decl(func);
        }

        if !program.functions.is_empty() {
            self.emitter.emit_line("");
        }

        self.generate_main_or_funcs(program, &runtime_globals);

        self.emitter.output.clone()
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
                    // Var globals are always runtime
                    runtime_global_names.insert(name.clone());
                }
                GlobalDeclaration::Const { name, value, .. } => {
                    if !self.compile_time_checker
                        .is_compile_time_constant_expr(value, &runtime_global_names) {
                        runtime_global_names.insert(name.clone());
                    }
                }
                GlobalDeclaration::Comptime { .. } => {}
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
                    // Var globals are always runtime
                    runtime_globals.push(global);
                }
                GlobalDeclaration::Comptime { name, var_type, value } => {
                    let inferred_type = if let Some(t) = var_type {
                        t.clone()
                    } else {
                        self.infer_expr_type(value)
                    };
                    self.symbol_table.insert(name.clone(), inferred_type);
                    comptime_globals.push(global);
                }
                GlobalDeclaration::Const { name, var_type, value } => {
                    let inferred_type = if let Some(t) = var_type {
                        t.clone()
                    } else {
                        self.infer_expr_type(value)
                    };
                    self.symbol_table.insert(name.clone(), inferred_type);

                    if runtime_global_names.contains(name) {
                        runtime_globals.push(global);
                    } else {
                        comptime_globals.push(global);
                    }
                }
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
            }
        }
    }

    /// Emits C header includes (freestanding).
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
    fn generate_main_or_funcs(&mut self, program: &Program,
                              runtime_globals: &[&GlobalDeclaration]) {
        let has_main = program.functions.iter().any(|f| f.name == "main");

        if !program.statements.is_empty() && !has_main {
            self.emit_synthetic_start(program, runtime_globals);
        } else {
            // Initialize runtime globals at the start of main (or before main is called)
            if has_main && !runtime_globals.is_empty() {
                self.emit_global_init_function(runtime_globals);
            }

            for func in &program.functions {
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
        self.emitter.emit_line("void _start(void) {");
        self.emitter.indent_level += 1;

        if has_global_init {
            self.emitter.emit_line("__init_globals();");
        }

        self.emitter.emit_line("int8_t exit_code = main();");
        self.emitter.emit_line("syscall1(SYS_exit, exit_code);");
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

        // Exit with code 0
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
                                                &self.function_signatures);
        type_inference.infer_expression_type(expr)
    }

    /// Maps a Summit type to its equivalent C type.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `type_name`: The Summit type name
    ///
    /// # Returns
    /// The equivalent C type as a string slice
    pub fn map_type(&self, type_name: &str) -> &str {
        self.type_mapper.map_type(type_name)
    }
}