use crate::frontend::ast::*;
use crate::analysis::type_checker::TypeChecker;
use std::collections::{HashMap, HashSet};
use super::function_analyzer::FunctionAnalyzer;
use super::statement_analyzer::StatementAnalyzer;
use super::scope_manager::ScopeManager;
use super::compile_time_checker::CompileTimeChecker;
use super::mutation_checker::MutationChecker;

/// Performs semantic analysis on a program.
///
/// # Parameters
/// - `program`: The program to analyze
///
/// # Returns
/// Ok(()) if analysis succeeds, Err with message on failure
pub fn analyze(program: &Program) -> Result<(), String> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(program)
}

/// Main semantic analyzer that does all analysis phases.
pub struct SemanticAnalyzer {
    pub imported_modules: HashSet<Vec<String>>,
    pub functions: HashMap<String, Function>,
    pub function_params: HashMap<String, Vec<(String, String)>>,
    pub structs: HashMap<String, StructDef>,
    pub enums: HashMap<String, EnumDef>,
    pub type_checker: TypeChecker,
    pub global_scope: HashMap<String, String>,
    pub global_mutability: HashMap<String, bool>,
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer.
    ///
    /// # Returns
    /// A new SemanticAnalyzer instance with empty state
    pub fn new() -> Self {
        SemanticAnalyzer {
            imported_modules: HashSet::new(),
            functions: HashMap::new(),
            function_params: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            type_checker: TypeChecker::new(),
            global_scope: HashMap::new(),
            global_mutability: HashMap::new(),
        }
    }

    /// Analyzes an entire program.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `program`: The program to analyze
    ///
    /// # Returns
    /// Ok(()) if analysis succeeds, Err with message on failure
    fn analyze_program(&mut self, program: &Program) -> Result<(), String> {
        for import in &program.imports {
            self.imported_modules.insert(import.path.clone());
        }

        for global in &program.globals {
            match global {
                GlobalDeclaration::Struct(struct_def) => {
                    if self.structs.contains_key(&struct_def.name) {
                        return Err(format!("Struct '{}' is defined multiple times", struct_def.name));
                    }

                    let mut field_names = HashSet::new();
                    for field in &struct_def.fields {
                        if !field_names.insert(&field.name) {
                            return Err(format!(
                                "Struct '{}' has duplicate field '{}'",
                                struct_def.name, field.name
                            ));
                        }
                    }

                    self.structs.insert(struct_def.name.clone(), struct_def.clone());
                }
                GlobalDeclaration::Enum(enum_def) => {
                    if self.enums.contains_key(&enum_def.name) {
                        return Err(format!("Enum '{}' is defined multiple times", enum_def.name));
                    }

                    let mut variant_names = HashSet::new();
                    for variant in &enum_def.variants {
                        if !variant_names.insert(&variant.name) {
                            return Err(format!(
                                "Enum '{}' has duplicate variant '{}'",
                                enum_def.name, variant.name
                            ));
                        }
                    }

                    self.enums.insert(enum_def.name.clone(), enum_def.clone());
                }
                _ => {}
            }
        }

        for global in &program.globals {
            self.analyze_global(global)?;
        }

        for stmt in &program.statements {
            match stmt {
                Statement::Var { name, var_type, value } => {
                    let scope_manager = ScopeManager::new(self);
                    scope_manager.analyze_expression(value, &self.global_scope)?;
                    let inferred_type = self.type_checker.infer_type(value, &self.global_scope,
                                                                     &self.functions, &self.structs)?;
                    self.validate_and_register_global(name, var_type, value, &inferred_type, true)?;
                }
                Statement::Const { name, var_type, value } => {
                    let scope_manager = ScopeManager::new(self);
                    scope_manager.analyze_expression(value, &self.global_scope)?;
                    let inferred_type = self.type_checker.infer_type(value, &self.global_scope,
                                                                     &self.functions, &self.structs)?;
                    self.validate_and_register_global(name, var_type, value, &inferred_type, false)?;
                }
                Statement::Comptime { name, var_type, value } => {
                    let compile_time_checker = CompileTimeChecker::new(self);
                    if !compile_time_checker.is_compile_time_constant(value) {
                        return Err(format!("Comptime global '{}' must be initialized with a compile-time constant", name));
                    }
                    let scope_manager = ScopeManager::new(self);
                    scope_manager.analyze_expression(value, &self.global_scope)?;
                    let inferred_type = self.type_checker.infer_type(value, &self.global_scope,
                                                                     &self.functions, &self.structs)?;
                    self.validate_and_register_global(name, var_type, value, &inferred_type, false)?;
                }
                _ => {}
            }
        }

        for func in &program.functions {
            if self.functions.contains_key(&func.name) {
                return Err(format!("Function '{}' is defined multiple times", func.name));
            }
            self.functions.insert(func.name.clone(), func.clone());

            let params: Vec<(String, String)> = func.params.iter()
                .map(|p| (p.name.clone(), p.param_type.clone()))
                .collect();
            self.function_params.insert(func.name.clone(), params);
        }

        let has_main = self.functions.contains_key("main");

        let actual_statements: Vec<&Statement> = program.statements.iter()
            .filter(|stmt| !matches!(stmt,
            Statement::Var { .. } |
            Statement::Const { .. } |
            Statement::Comptime { .. }
        ))
            .collect();

        if has_main && !actual_statements.is_empty() {
            return Err("Cannot have top-level executable statements when a 'main' function is defined"
                .to_string());
        }

        if !has_main && actual_statements.is_empty() && program.globals.is_empty() &&
            program.statements.iter().all(|s| matches!(s,
           Statement::Var { .. } |
           Statement::Const { .. } |
           Statement::Comptime { .. }
       )) {
            return Err("No 'main' function defined and no top-level statements".to_string());
        }

        if !actual_statements.is_empty() {
            let mut mutations = HashSet::new();
            let mut var_declarations = HashMap::new();
            let mutation_checker = MutationChecker::new();

            for stmt in &actual_statements {
                mutation_checker.collect_mutations(stmt, &mut mutations, &mut var_declarations)?;
            }

            for (name, is_mutable) in &self.global_mutability {
                if !var_declarations.contains_key(name) {
                    var_declarations.insert(name.clone(), *is_mutable);
                }
            }

            mutation_checker.validate_mutations(&mutations, &var_declarations)?;
        }

        if !actual_statements.is_empty() && !has_main {
            let mut scope = self.global_scope.clone();
            let mut mutability = self.global_mutability.clone();
            let stmt_analyzer = StatementAnalyzer::new(self);
            for stmt in &actual_statements {
                stmt_analyzer.analyze_stmt(stmt, &mut scope, &mut mutability)?;
            }
        }

        let func_analyzer = FunctionAnalyzer::new(self);
        for func in &program.functions {
            func_analyzer.analyze_func(func)?;
        }

        Ok(())
    }

    /// Analyzes a global declaration.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `global`: The global declaration to analyze
    ///
    /// # Returns
    /// Ok(()) if analysis succeeds, Err with message on failure
    fn analyze_global(&mut self, global: &GlobalDeclaration) -> Result<(), String> {
        let compile_time_checker = CompileTimeChecker::new(self);

        match global {
            GlobalDeclaration::Var { name, var_type, value } => {
                let scope_manager = ScopeManager::new(self);
                scope_manager.analyze_expression(value, &self.global_scope)?;
                let inferred_type = self.type_checker.infer_type(value, &self.global_scope,
                                                                 &self.functions, &self.structs)?;
                self.validate_and_register_global(name, var_type, value, &inferred_type, true)?;
                Ok(())
            }
            GlobalDeclaration::Const { name, var_type, value } => {
                let scope_manager = ScopeManager::new(self);
                scope_manager.analyze_expression(value, &self.global_scope)?;
                let inferred_type = self.type_checker.infer_type(value, &self.global_scope,
                                                                 &self.functions, &self.structs)?;
                self.validate_and_register_global(name, var_type, value, &inferred_type, false)?;
                Ok(())
            }
            GlobalDeclaration::Comptime { name, var_type, value } => {
                if !compile_time_checker.is_compile_time_constant(value) {
                    return Err(format!("Comptime global '{}' must be initialized with a compile-time constant", name));
                }

                let scope_manager = ScopeManager::new(self);
                scope_manager.analyze_expression(value, &self.global_scope)?;
                let inferred_type = self.type_checker.infer_type(value, &self.global_scope,
                                                                 &self.functions, &self.structs)?;
                self.validate_and_register_global(name, var_type, value, &inferred_type, false)?;
                Ok(())
            }
            GlobalDeclaration::Struct(_) => {
                Ok(())
            }
            GlobalDeclaration::Enum(_) => {
                Ok(())
            }
        }
    }

    /// Validates type compatibility and registers a global variable in the scope.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `name`: The variable name
    /// - `var_type`: Optional explicit type annotation
    /// - `value`: The initialization expression
    /// - `inferred_type`: The type inferred from the expression
    /// - `is_mutable`: Whether the variable is mutable
    ///
    /// # Returns
    /// Ok(()) if validation succeeds, Err with message on failure
    fn validate_and_register_global(&mut self, name: &str, var_type: &Option<String>,
                                    value: &Expression, inferred_type: &str,
                                    is_mutable: bool) -> Result<(), String> {
        if let Some(explicit_type) = var_type {
            self.type_checker.check_type_compatibility(explicit_type, inferred_type, value)?;
            self.type_checker.check_expression_bounds(value, explicit_type)?;
            self.global_scope.insert(name.to_string(), explicit_type.clone());
        } else {
            self.type_checker.check_expression_bounds(value, inferred_type)?;
            self.global_scope.insert(name.to_string(), inferred_type.to_string());
        }

        self.global_mutability.insert(name.to_string(), is_mutable);

        Ok(())
    }
}