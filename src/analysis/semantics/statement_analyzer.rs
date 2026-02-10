use crate::frontend::ast::*;
use std::collections::HashMap;
use super::analyzer::SemanticAnalyzer;
use super::expression_analyzer::ExpressionAnalyzer;
use super::module_checker::ModuleChecker;
use super::compile_time_checker::CompileTimeChecker;
use crate::utils::io_path_matcher::IoPathMatcher;

/// Analyzes statements in the AST for semantic correctness.
pub struct StatementAnalyzer<'a> {
    /// The main program analyzer checking for semantic correctness
    analyzer: &'a SemanticAnalyzer,
}

impl<'a> StatementAnalyzer<'a> {
    /// Creates a new statement analyzer.
    ///
    /// # Parameters
    /// - `analyzer`: Reference to the semantic analyzer
    ///
    /// # Returns
    /// A new StatementAnalyzer instance
    pub fn new(analyzer: &'a SemanticAnalyzer) -> Self {
        StatementAnalyzer { analyzer }
    }

    /// Analyzes a statement with expected return type checking.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `stmt`: The statement to analyze
    /// - `scope`: Current variable scope
    /// - `func_name`: Name of the containing function
    /// - `expected_return_type`: The function's return type
    ///
    /// # Returns
    /// Ok(()) if analysis succeeds, Err with message on failure
    pub fn analyze_stmt_with_return_type(&self, stmt: &Statement,
                                         scope: &mut HashMap<String, String>,
                                         func_name: &str,
                                         expected_return_type: &str) -> Result<(), String> {
        match stmt {
            Statement::When { value, cases, else_block } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(value, scope)?;

                let value_type = self.analyzer.type_checker.infer_type(value, scope,
                                                                       &self.analyzer.functions)?;

                for case in cases {
                    match &case.pattern {
                        WhenPattern::Single(pattern_expr) => {
                            expr_analyzer.analyze_expr(pattern_expr, scope)?;
                            let pattern_type = self.analyzer.type_checker.infer_type(pattern_expr,
                                                                                     scope,
                                                                                     &self.analyzer.functions)?;

                            if !self.analyzer.type_checker.types_compatible(&value_type, &pattern_type) {
                                return Err(format!(
                                    "When case pattern has incompatible type '{}', expected '{}'",
                                    pattern_type, value_type
                                ));
                            }
                        }
                        WhenPattern::Range { start, end, .. } => {
                            expr_analyzer.analyze_expr(start, scope)?;
                            expr_analyzer.analyze_expr(end, scope)?;

                            let start_type = self.analyzer.type_checker.infer_type(start,
                                                                                   scope,
                                                                                   &self.analyzer.functions)?;
                            let end_type = self.analyzer.type_checker.infer_type(end,
                                                                                 scope,
                                                                                 &self.analyzer.functions)?;

                            if !self.analyzer.type_checker.types_compatible(&value_type, &start_type) {
                                return Err(format!(
                                    "When case range start has incompatible type '{}', expected '{}'",
                                    start_type, value_type
                                ));
                            }

                            if !self.analyzer.type_checker.types_compatible(&value_type, &end_type) {
                                return Err(format!(
                                    "When case range end has incompatible type '{}', expected '{}'",
                                    end_type, value_type
                                ));
                            }
                        }
                    }

                    let mut case_scope = scope.clone();
                    for stmt in &case.body {
                        self.analyze_stmt_with_return_type(stmt, &mut case_scope, func_name,
                                                           expected_return_type)?;
                    }
                }

                if let Some(else_stmts) = else_block {
                    let mut else_scope = scope.clone();
                    for stmt in else_stmts {
                        self.analyze_stmt_with_return_type(stmt, &mut else_scope, func_name,
                                                           expected_return_type)?;
                    }
                }

                Ok(())
            }
            Statement::Expect { condition, pattern, else_block } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(condition, scope)?;

                if let Some(p) = pattern {
                    let condition_type = self.analyzer.type_checker.infer_type(condition, scope,
                                                                               &self.analyzer.functions)?;

                    match p {
                        ExpectPattern::Single(pattern_expr) => {
                            expr_analyzer.analyze_expr(pattern_expr, scope)?;
                            let pattern_type = self.analyzer.type_checker.infer_type(pattern_expr,
                                                                                     scope,
                                                                                     &self.analyzer.functions)?;

                            if !self.analyzer.type_checker.types_compatible(&condition_type, &pattern_type) {
                                return Err(format!(
                                    "Expect pattern has incompatible type '{}', expected '{}'",
                                    pattern_type, condition_type
                                ));
                            }
                        }
                        ExpectPattern::Range { start, end, .. } => {
                            expr_analyzer.analyze_expr(start, scope)?;
                            expr_analyzer.analyze_expr(end, scope)?;

                            let start_type = self.analyzer.type_checker.infer_type(start,
                                                                                   scope,
                                                                                   &self.analyzer.functions)?;
                            let end_type = self.analyzer.type_checker.infer_type(end,
                                                                                 scope,
                                                                                 &self.analyzer.functions)?;

                            if !self.analyzer.type_checker.types_compatible(&condition_type, &start_type) {
                                return Err(format!(
                                    "Expect range start has incompatible type '{}', expected '{}'",
                                    start_type, condition_type
                                ));
                            }

                            if !self.analyzer.type_checker.types_compatible(&condition_type, &end_type) {
                                return Err(format!(
                                    "Expect range end has incompatible type '{}', expected '{}'",
                                    end_type, condition_type
                                ));
                            }
                        }
                    }
                }

                let mut else_scope = scope.clone();
                for stmt in else_block {
                    self.analyze_stmt_with_return_type(stmt, &mut else_scope, func_name,
                                                       expected_return_type)?;
                }

                Ok(())
            }
            Statement::For { variable, start, end,
                step, filter, body, .. } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(start, scope)?;
                expr_analyzer.analyze_expr(end, scope)?;

                let start_type = self.analyzer.type_checker.infer_type(start, scope,
                                                                       &self.analyzer.functions)?;
                let end_type = self.analyzer.type_checker.infer_type(end, scope,
                                                                     &self.analyzer.functions)?;

                if let Some(step_expr) = step {
                    expr_analyzer.analyze_expr(step_expr, scope)?;
                }

                let mut body_scope = scope.clone();
                let loop_var_type = self.analyzer.type_checker.wider_type(&start_type,
                                                                          &end_type);
                body_scope.insert(variable.clone(), loop_var_type);

                if let Some(filter_expr) = filter {
                    expr_analyzer.analyze_expr(filter_expr, &body_scope)?;
                }

                for stmt in body {
                    self.analyze_stmt_with_return_type(stmt, &mut body_scope, func_name,
                                                       expected_return_type)?;
                }

                Ok(())
            }
            Statement::Return(expr) => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(expr, scope)?;
                let actual_return_type = self.analyzer.type_checker
                    .infer_type(expr, scope, &self.analyzer.functions)?;

                if actual_return_type == expected_return_type {
                    return Ok(());
                }

                if self.analyzer.type_checker.can_convert(&actual_return_type,
                                                          expected_return_type) {
                    return Ok(());
                }

                if self.analyzer.type_checker.is_signed(&actual_return_type)
                    && !self.analyzer.type_checker.is_signed(expected_return_type) {
                    let actual_size = self.analyzer.type_checker
                        .get_type_size(&actual_return_type);
                    let expected_size = self.analyzer.type_checker
                        .get_type_size(expected_return_type);

                    if actual_size < expected_size {
                        if let Some(val) = self.analyzer.type_checker
                            .get_literal_value(expr) {
                            if val > self.analyzer.type_checker
                                .get_unsigned_max(expected_return_type) {
                                return Err(format!(
                                    "Function '{}': cannot return value {} as type '{}' (exceeds maximum: {})",
                                    func_name, val, expected_return_type,
                                    self.analyzer.type_checker
                                        .get_unsigned_max(expected_return_type)
                                ));
                            }
                        }
                        return Ok(());
                    }
                }

                if !self.analyzer.type_checker.is_signed(&actual_return_type)
                    && self.analyzer.type_checker.is_signed(expected_return_type) {
                    let actual_size = self.analyzer.type_checker
                        .get_type_size(&actual_return_type);
                    let expected_size = self.analyzer.type_checker
                        .get_type_size(expected_return_type);

                    if actual_size < expected_size {
                        return Ok(());
                    }
                }

                if self.analyzer.type_checker.would_truncate(&actual_return_type,
                                                             expected_return_type) {
                    return Err(format!(
                        "Function '{}': return type mismatch. Expression has type '{}' but function expects return type '{}' (would lose data)",
                        func_name, actual_return_type, expected_return_type
                    ));
                }

                if !self.analyzer.type_checker.types_compatible(&actual_return_type,
                                                                expected_return_type) {
                    return Err(format!(
                        "Function '{}': return type mismatch. Expected '{}', got '{}'",
                        func_name, expected_return_type, actual_return_type
                    ));
                }

                Err(format!(
                    "Function '{}': return type mismatch. Cannot implicitly convert '{}' to '{}' (same size but different signedness)",
                    func_name, actual_return_type, expected_return_type
                ))
            }
            Statement::If { condition, then_block,
                elseif_blocks, else_block } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(condition, scope)?;

                let mut then_scope = scope.clone();
                for stmt in then_block {
                    self.analyze_stmt_with_return_type(stmt, &mut then_scope, func_name,
                                                       expected_return_type)?;
                }

                for elseif_block in elseif_blocks {
                    expr_analyzer.analyze_expr(&elseif_block.condition, scope)?;
                    let mut elseif_scope = scope.clone();
                    for stmt in &elseif_block.body {
                        self.analyze_stmt_with_return_type(stmt, &mut elseif_scope, func_name,
                                                           expected_return_type)?;
                    }
                }

                if let Some(else_stmts) = else_block {
                    let mut else_scope = scope.clone();
                    for stmt in else_stmts {
                        self.analyze_stmt_with_return_type(stmt, &mut else_scope, func_name,
                                                           expected_return_type)?;
                    }
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(condition, scope)?;

                let mut body_scope = scope.clone();
                for stmt in body {
                    self.analyze_stmt_with_return_type(stmt, &mut body_scope, func_name,
                                                       expected_return_type)?;
                }
                Ok(())
            }
            Statement::Next => {
                Ok(())
            }
            Statement::Stop => {
                Ok(())
            }
            Statement::Fallthrough => {
                Ok(())
            }
            _ => {
                self.analyze_stmt(stmt, scope)
            }
        }
    }

    /// Analyzes a statement for semantic correctness.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `stmt`: The statement to analyze
    /// - `scope`: Current variable scope
    ///
    /// # Returns
    /// Ok(()) if analysis succeeds, Err with message on failure
    pub fn analyze_stmt(&self, stmt: &Statement,
                        scope: &mut HashMap<String, String>) -> Result<(), String> {
        match stmt {
            Statement::When { value, cases, else_block } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(value, scope)?;

                let value_type = self.analyzer.type_checker.infer_type(value, scope,
                                                                       &self.analyzer.functions)?;

                for case in cases {
                    match &case.pattern {
                        WhenPattern::Single(pattern_expr) => {
                            expr_analyzer.analyze_expr(pattern_expr, scope)?;
                            let pattern_type = self.analyzer.type_checker.infer_type(pattern_expr,
                                                                                     scope,
                                                                                     &self.analyzer.functions)?;

                            if !self.analyzer.type_checker.types_compatible(&value_type, &pattern_type) {
                                return Err(format!(
                                    "When case pattern has incompatible type '{}', expected '{}'",
                                    pattern_type, value_type
                                ));
                            }
                        }
                        WhenPattern::Range { start, end, .. } => {
                            expr_analyzer.analyze_expr(start, scope)?;
                            expr_analyzer.analyze_expr(end, scope)?;

                            let start_type = self.analyzer.type_checker.infer_type(start,
                                                                                   scope,
                                                                                   &self.analyzer.functions)?;
                            let end_type = self.analyzer.type_checker.infer_type(end,
                                                                                 scope,
                                                                                 &self.analyzer.functions)?;

                            if !self.analyzer.type_checker.types_compatible(&value_type, &start_type) {
                                return Err(format!(
                                    "When case range start has incompatible type '{}', expected '{}'",
                                    start_type, value_type
                                ));
                            }

                            if !self.analyzer.type_checker.types_compatible(&value_type, &end_type) {
                                return Err(format!(
                                    "When case range end has incompatible type '{}', expected '{}'",
                                    end_type, value_type
                                ));
                            }
                        }
                    }

                    let mut case_scope = scope.clone();
                    for stmt in &case.body {
                        self.analyze_stmt(stmt, &mut case_scope)?;
                    }
                }

                if let Some(else_stmts) = else_block {
                    let mut else_scope = scope.clone();
                    for stmt in else_stmts {
                        self.analyze_stmt(stmt, &mut else_scope)?;
                    }
                }

                Ok(())
            }
            Statement::Expect { condition, pattern, else_block } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(condition, scope)?;

                if let Some(p) = pattern {
                    let condition_type = self.analyzer.type_checker.infer_type(condition, scope,
                                                                               &self.analyzer.functions)?;

                    match p {
                        ExpectPattern::Single(pattern_expr) => {
                            expr_analyzer.analyze_expr(pattern_expr, scope)?;
                            let pattern_type = self.analyzer.type_checker.infer_type(pattern_expr,
                                                                                     scope,
                                                                                     &self.analyzer.functions)?;

                            if !self.analyzer.type_checker.types_compatible(&condition_type, &pattern_type) {
                                return Err(format!(
                                    "Expect pattern has incompatible type '{}', expected '{}'",
                                    pattern_type, condition_type
                                ));
                            }
                        }
                        ExpectPattern::Range { start, end, .. } => {
                            expr_analyzer.analyze_expr(start, scope)?;
                            expr_analyzer.analyze_expr(end, scope)?;

                            let start_type = self.analyzer.type_checker.infer_type(start,
                                                                                   scope,
                                                                                   &self.analyzer.functions)?;
                            let end_type = self.analyzer.type_checker.infer_type(end,
                                                                                 scope,
                                                                                 &self.analyzer.functions)?;

                            if !self.analyzer.type_checker.types_compatible(&condition_type, &start_type) {
                                return Err(format!(
                                    "Expect range start has incompatible type '{}', expected '{}'",
                                    start_type, condition_type
                                ));
                            }

                            if !self.analyzer.type_checker.types_compatible(&condition_type, &end_type) {
                                return Err(format!(
                                    "Expect range end has incompatible type '{}', expected '{}'",
                                    end_type, condition_type
                                ));
                            }
                        }
                    }
                }

                let mut else_scope = scope.clone();
                for stmt in else_block {
                    self.analyze_stmt(stmt, &mut else_scope)?;
                }

                Ok(())
            }
            Statement::For { variable, start, end,
                step, filter, body, .. } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(start, scope)?;
                expr_analyzer.analyze_expr(end, scope)?;

                let start_type = self.analyzer.type_checker.infer_type(start,
                                                                       scope,
                                                                       &self.analyzer.functions)?;
                let end_type = self.analyzer.type_checker.infer_type(end,
                                                                     scope,
                                                                     &self.analyzer.functions)?;

                let module_checker = ModuleChecker::new(self.analyzer);
                if !module_checker.is_integer_type(&start_type) {
                    return Err(format!("For loop start must be an integer type, got '{}'",
                                       start_type));
                }
                if !module_checker.is_integer_type(&end_type) {
                    return Err(format!("For loop end must be an integer type, got '{}'", end_type));
                }

                if let Some(step_expr) = step {
                    expr_analyzer.analyze_expr(step_expr, scope)?;
                    let step_type = self.analyzer.type_checker.infer_type(step_expr,
                                                                          scope,
                                                                          &self.analyzer.functions)?;
                    if !module_checker.is_integer_type(&step_type) {
                        return Err(format!("For loop step must be an integer type, got '{}'",
                                           step_type));
                    }
                }

                let mut body_scope = scope.clone();
                let loop_var_type = self.analyzer.type_checker.wider_type(&start_type,
                                                                          &end_type);
                body_scope.insert(variable.clone(), loop_var_type);

                if let Some(filter_expr) = filter {
                    expr_analyzer.analyze_expr(filter_expr, &body_scope)?;
                }

                for stmt in body {
                    self.analyze_stmt(stmt, &mut body_scope)?;
                }

                Ok(())
            }
            Statement::Var { name, var_type, value } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(value, scope)?;
                let inferred_type = self.analyzer.type_checker.infer_type(value, scope,
                                                                          &self.analyzer.functions)?;

                if let Some(explicit_type) = var_type {
                    if let Expression::Call { path, type_args, .. } = value {
                        if IoPathMatcher::is_read(path) {
                            if let Some(types) = type_args {
                                if types.len() == 1 {
                                    let read_type = &types[0];
                                    if explicit_type != read_type {
                                        return Err(format!(
                                            "Type mismatch: variable '{}' has type '{}' but io::read<{}> returns '{}'",
                                            name, explicit_type, read_type, read_type
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    self.analyzer.type_checker.check_type_compatibility(explicit_type,
                                                                        &inferred_type, value)?;
                    self.analyzer.type_checker.check_expression_bounds(value, explicit_type)?;
                    scope.insert(name.clone(), explicit_type.clone());
                } else {
                    self.analyzer.type_checker.check_expression_bounds(value, &inferred_type)?;
                    scope.insert(name.clone(), inferred_type.clone());
                }
                Ok(())
            }
            Statement::Const { name, var_type, value } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(value, scope)?;
                let inferred_type = self.analyzer.type_checker.infer_type(value, scope,
                                                                          &self.analyzer.functions)?;

                if let Some(explicit_type) = var_type {
                    if let Expression::Call { path, type_args, .. } = value {
                        if IoPathMatcher::is_read(path) {
                            if let Some(types) = type_args {
                                if types.len() == 1 {
                                    let read_type = &types[0];
                                    if explicit_type != read_type {
                                        return Err(format!(
                                            "Type mismatch: constant '{}' has type '{}' but io::read<{}> returns '{}'",
                                            name, explicit_type, read_type, read_type
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    self.analyzer.type_checker.check_type_compatibility(explicit_type,
                                                                        &inferred_type, value)?;
                    self.analyzer.type_checker.check_expression_bounds(value, explicit_type)?;
                    scope.insert(name.clone(), explicit_type.clone());
                } else {
                    self.analyzer.type_checker.check_expression_bounds(value, &inferred_type)?;
                    scope.insert(name.clone(), inferred_type.clone());
                }
                Ok(())
            }
            Statement::Comptime { name, var_type, value } => {
                let compile_time_checker = CompileTimeChecker::new(self.analyzer);
                if !compile_time_checker.is_compile_time_evaluable(value) {
                    return Err(format!("Comptime '{}' must be evaluable at compile time", name));
                }

                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(value, scope)?;
                let inferred_type = self.analyzer.type_checker
                    .infer_type(value, scope, &self.analyzer.functions)?;
                if let Some(explicit_type) = var_type {
                    if let Expression::Call { path, type_args, .. } = value {
                        if IoPathMatcher::is_read(path) {
                            if let Some(types) = type_args {
                                if types.len() == 1 {
                                    let read_type = &types[0];
                                    if explicit_type != read_type {
                                        return Err(format!(
                                            "Type mismatch: comptime variable '{}' has type '{}' but io::read<{}> returns '{}'",
                                            name, explicit_type, read_type, read_type
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    self.analyzer.type_checker
                        .check_type_compatibility(explicit_type, &inferred_type, value)?;
                    self.analyzer.type_checker.check_expression_bounds(value, explicit_type)?;
                    scope.insert(name.clone(), explicit_type.clone());
                } else {
                    self.analyzer.type_checker.check_expression_bounds(value, &inferred_type)?;
                    scope.insert(name.clone(), inferred_type.clone());
                }
                Ok(())
            }
            Statement::Assign { name, value } => {
                if !scope.contains_key(name) {
                    return Err(format!("Cannot assign to undefined variable: {}", name));
                }

                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(value, scope)?;
                let var_type = scope.get(name).unwrap().clone();
                let value_type = self.analyzer.type_checker.infer_type(value, scope,
                                                                       &self.analyzer.functions)?;
                self.analyzer.type_checker.check_type_compatibility(&var_type, &value_type, value)?;

                Ok(())
            }
            Statement::Return(expr) => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(expr, scope)
            }
            Statement::Expression(expr) => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(expr, scope)
            }
            Statement::If { condition, then_block,
                elseif_blocks, else_block } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(condition, scope)?;

                let mut then_scope = scope.clone();
                for stmt in then_block {
                    self.analyze_stmt(stmt, &mut then_scope)?;
                }

                for elseif_block in elseif_blocks {
                    expr_analyzer.analyze_expr(&elseif_block.condition, scope)?;
                    let mut elseif_scope = scope.clone();
                    for stmt in &elseif_block.body {
                        self.analyze_stmt(stmt, &mut elseif_scope)?;
                    }
                }

                if let Some(else_stmts) = else_block {
                    let mut else_scope = scope.clone();
                    for stmt in else_stmts {
                        self.analyze_stmt(stmt, &mut else_scope)?;
                    }
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                let expr_analyzer = ExpressionAnalyzer::new(self.analyzer);
                expr_analyzer.analyze_expr(condition, scope)?;

                let mut body_scope = scope.clone();
                for stmt in body {
                    self.analyze_stmt(stmt, &mut body_scope)?;
                }
                Ok(())
            }
            Statement::Next => {
                Ok(())
            }
            Statement::Stop => {
                Ok(())
            }
            Statement::Fallthrough => {
                Ok(())
            }
        }
    }
}