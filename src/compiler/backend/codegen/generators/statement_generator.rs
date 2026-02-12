use crate::frontend::ast::*;
use super::program_generator::ProgramGenerator;
use super::expression_generator::ExpressionGenerator;
use crate::utils::type_resolver::TypeResolver;
use super::super::helpers::{TypeInference};

/// Generates C code for Summit statements.
pub struct StatementGenerator<'a> {
    /// The Program Generator responsible for generating the executable program
    generator: &'a mut ProgramGenerator,
}

impl<'a> StatementGenerator<'a> {
    /// Creates a new StatementGenerator instance.
    ///
    /// # Parameters
    /// - `generator`: The parent program generator
    pub fn new(generator: &'a mut ProgramGenerator) -> Self {
        StatementGenerator { generator }
    }

    /// Generates C code for a Summit statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `stmt`: The statement to generate code for
    pub fn generate_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::When { value, cases, else_block } => {
                self.emit_when_stmt(value, cases, else_block);
            }
            Statement::Expect { condition, pattern, else_block } => {
                self.emit_expect_stmt(condition, pattern, else_block);
            }
            Statement::For { variable, start, end, inclusive,
                step, filter, body } => {
                self.emit_for_loop(variable, start, end, *inclusive, step, filter, body);
            }
            Statement::Var { name, var_type, value } => {
                self.emit_let_stmt(name, var_type, value);
            }
            Statement::Const { name, var_type, value } => {
                self.emit_const_stmt(name, var_type, value);
            }
            Statement::Comptime { name, var_type, value } => {
                self.emit_comptime_stmt(name, var_type, value);
            }
            Statement::Assign { name, value } => {
                self.emit_assign_stmt(name, value);
            }
            Statement::FieldAssign { object, field, value } => {
                self.emit_field_assign_stmt(object, field, value);
            }
            Statement::Return(expr) => {
                self.emit_ret_stmt(expr);
            }
            Statement::Expression(expr) => {
                self.emit_expr_stmt(expr);
            }
            Statement::If { condition, then_block,
                elseif_blocks, else_block } => {
                self.emit_if_stmt(condition, then_block, elseif_blocks, else_block);
            }
            Statement::While { condition, body } => {
                self.emit_while_stmt(condition, body);
            }
            Statement::Next => {
                self.emit_next_stmt();
            }
            Statement::Stop => {
                self.emit_stop_stmt();
            }
            Statement::Fallthrough => {
                self.emit_fallthrough_stmt();
            }
        }
    }

    /// Generates C code for an expect statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `condition`: The condition expression
    /// - `pattern`: Optional pattern to match against
    /// - `else_block`: Statements to execute if expectation fails
    fn emit_expect_stmt(&mut self, condition: &Expression, pattern: &Option<ExpectPattern>,
                        else_block: &[Statement]) {
        self.generator.emitter.indent();

        if let Some(p) = pattern {
            match p {
                ExpectPattern::Single(expr) => {
                    self.generator.emitter.emit("if (!(");
                    let mut expr_gen = ExpressionGenerator::new(self.generator);
                    expr_gen.generate_expr(condition);
                    self.generator.emitter.emit(" == ");
                    let mut expr_gen = ExpressionGenerator::new(self.generator);
                    expr_gen.generate_expr(expr);
                    self.generator.emitter.emit(")) {\n");
                }
                ExpectPattern::Range { start, end, inclusive } => {
                    self.generator.emitter.emit("if (!(");
                    let mut expr_gen = ExpressionGenerator::new(self.generator);
                    expr_gen.generate_expr(condition);
                    self.generator.emitter.emit(" >= ");
                    let mut expr_gen = ExpressionGenerator::new(self.generator);
                    expr_gen.generate_expr(start);

                    self.generator.emitter.emit(" && ");

                    let mut expr_gen = ExpressionGenerator::new(self.generator);
                    expr_gen.generate_expr(condition);

                    if *inclusive {
                        self.generator.emitter.emit(" <= ");
                    } else {
                        self.generator.emitter.emit(" < ");
                    }

                    let mut expr_gen = ExpressionGenerator::new(self.generator);
                    expr_gen.generate_expr(end);
                    self.generator.emitter.emit(")) {\n");
                }
            }
        } else {
            self.generator.emitter.emit("if (!(");
            let mut expr_gen = ExpressionGenerator::new(self.generator);
            expr_gen.generate_expr(condition);
            self.generator.emitter.emit(")) {\n");
        }

        self.generator.emitter.indent_level += 1;

        for stmt in else_block {
            self.generate_stmt(stmt);
        }

        self.generator.emitter.indent_level -= 1;
        self.generator.emitter.emit_line("}");
    }

    /// Generates C code for a when statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `value`: The value to switch on
    /// - `cases`: The when cases
    /// - `else_block`: Optional default case
    fn emit_when_stmt(&mut self, value: &Expression, cases: &[WhenCase],
                      else_block: &Option<Vec<Statement>>) {
        let has_ranges = cases.iter().any(|case| matches!(case.pattern, WhenPattern::Range { .. }));

        if has_ranges {
            self.emit_when_with_ranges(value, cases, else_block);
        } else {
            self.emit_when_with_switch(value, cases, else_block);
        }
    }

    /// Generates C code for when statement using if-else chain for range patterns.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `value`: The value to match against
    /// - `cases`: The when cases
    /// - `else_block`: Optional default case
    fn emit_when_with_ranges(&mut self, value: &Expression, cases: &[WhenCase],
                             else_block: &Option<Vec<Statement>>) {
        let has_any_fallthrough = cases.iter().any(|case| {
            if let Some(last_stmt) = case.body.last() {
                matches!(last_stmt, Statement::Fallthrough)
            } else {
                false
            }
        });

        if has_any_fallthrough {
            self.generator.emitter.indent();
            self.generator.emitter.emit("{\n");
            self.generator.emitter.indent_level += 1;

            self.generator.emitter.indent();
            let type_inference = TypeInference::new(&self.generator.symbol_table,
                                                    &self.generator.function_signatures,
                                                    &self.generator.struct_defs);
            let value_type = type_inference.infer_expression_type(value);
            let c_type = self.generator.map_type(&value_type).to_string();

            self.generator.emitter.emit(&format!("{} __when_value = ", c_type));
            let mut expr_gen = ExpressionGenerator::new(self.generator);
            expr_gen.generate_expr(value);
            self.generator.emitter.emit(";\n");

            self.generator.emitter.indent();
            self.generator.emitter.emit("int __when_matched = 0;\n");

            for (i, case) in cases.iter().enumerate() {
                self.generator.emitter.indent();
                self.generator.emitter.emit(&format!("__when_case_{}:\n", i));

                self.generator.emitter.indent();
                self.generator.emitter.emit("if (__when_matched || (");

                match &case.pattern {
                    WhenPattern::Single(expr) => {
                        self.generator.emitter.emit("__when_value == ");
                        let mut expr_gen = ExpressionGenerator::new(self.generator);
                        expr_gen.generate_expr(expr);
                    }
                    WhenPattern::Range { start, end, inclusive } => {
                        self.generator.emitter.emit("__when_value >= ");
                        let mut expr_gen = ExpressionGenerator::new(self.generator);
                        expr_gen.generate_expr(start);

                        self.generator.emitter.emit(" && __when_value ");
                        if *inclusive {
                            self.generator.emitter.emit("<= ");
                        } else {
                            self.generator.emitter.emit("< ");
                        }

                        let mut expr_gen = ExpressionGenerator::new(self.generator);
                        expr_gen.generate_expr(end);
                    }
                }

                self.generator.emitter.emit(")) {\n");
                self.generator.emitter.indent_level += 1;

                let has_fallthrough = if let Some(last_stmt) = case.body.last() {
                    matches!(last_stmt, Statement::Fallthrough)
                } else {
                    false
                };

                for stmt in &case.body {
                    if !matches!(stmt, Statement::Fallthrough) {
                        self.generate_stmt(stmt);
                    }
                }

                if has_fallthrough {
                    self.generator.emitter.indent();
                    self.generator.emitter.emit("__when_matched = 1;\n");
                    if i + 1 < cases.len() {
                        self.generator.emitter.indent();
                        self.generator.emitter.emit(&format!("goto __when_case_{};\n", i + 1));
                    }
                } else {
                    self.generator.emitter.indent();
                    self.generator.emitter.emit("__when_matched = 1;\n");
                }

                self.generator.emitter.indent_level -= 1;
                self.generator.emitter.indent();
                self.generator.emitter.emit("}\n");
            }

            if let Some(else_stmts) = else_block {
                self.generator.emitter.indent();
                self.generator.emitter.emit("if (!__when_matched) {\n");

                self.generator.emitter.indent_level += 1;

                for stmt in else_stmts {
                    self.generate_stmt(stmt);
                }

                self.generator.emitter.indent_level -= 1;
                self.generator.emitter.indent();
                self.generator.emitter.emit("}\n");
            }

            self.generator.emitter.indent_level -= 1;
            self.generator.emitter.emit_line("}");
        } else {
            let mut first = true;

            for case in cases {
                self.generator.emitter.indent();

                if first {
                    self.generator.emitter.emit("if (");
                    first = false;
                } else {
                    self.generator.emitter.emit("else if (");
                }

                match &case.pattern {
                    WhenPattern::Single(expr) => {
                        let mut expr_gen = ExpressionGenerator::new(self.generator);
                        expr_gen.generate_expr(value);
                        self.generator.emitter.emit(" == ");
                        let mut expr_gen = ExpressionGenerator::new(self.generator);
                        expr_gen.generate_expr(expr);
                    }
                    WhenPattern::Range { start, end, inclusive } => {
                        let mut expr_gen = ExpressionGenerator::new(self.generator);
                        expr_gen.generate_expr(value);
                        self.generator.emitter.emit(" >= ");
                        let mut expr_gen = ExpressionGenerator::new(self.generator);
                        expr_gen.generate_expr(start);

                        self.generator.emitter.emit(" && ");

                        let mut expr_gen = ExpressionGenerator::new(self.generator);
                        expr_gen.generate_expr(value);

                        if *inclusive {
                            self.generator.emitter.emit(" <= ");
                        } else {
                            self.generator.emitter.emit(" < ");
                        }

                        let mut expr_gen = ExpressionGenerator::new(self.generator);
                        expr_gen.generate_expr(end);
                    }
                }

                self.generator.emitter.emit(") {\n");
                self.generator.emitter.indent_level += 1;

                for stmt in &case.body {
                    self.generate_stmt(stmt);
                }

                self.generator.emitter.indent_level -= 1;
                self.generator.emitter.indent();
                self.generator.emitter.emit("}\n");
            }

            if let Some(else_stmts) = else_block {
                self.generator.emitter.indent();
                self.generator.emitter.emit("else {\n");

                self.generator.emitter.indent_level += 1;

                for stmt in else_stmts {
                    self.generate_stmt(stmt);
                }

                self.generator.emitter.indent_level -= 1;
                self.generator.emitter.emit_line("}");
            }
        }
    }

    /// Generates C code for when statement using switch for single-value patterns only.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `value`: The value to switch on
    /// - `cases`: The when cases
    /// - `else_block`: Optional default case
    fn emit_when_with_switch(&mut self, value: &Expression, cases: &[WhenCase],
                             else_block: &Option<Vec<Statement>>) {
        self.generator.emitter.indent();
        self.generator.emitter.emit("switch (");

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(value);

        self.generator.emitter.emit(") {\n");
        self.generator.emitter.indent_level += 1;

        for case in cases {
            self.generator.emitter.indent();
            self.generator.emitter.emit("case ");

            if let WhenPattern::Single(expr) = &case.pattern {
                let mut expr_gen = ExpressionGenerator::new(self.generator);
                expr_gen.generate_expr(expr);
            }

            self.generator.emitter.emit(":\n");
            self.generator.emitter.indent_level += 1;

            let has_fallthrough = if let Some(last_stmt) = case.body.last() {
                matches!(last_stmt, Statement::Fallthrough)
            } else {
                false
            };

            for stmt in &case.body {
                self.generate_stmt(stmt);
            }

            if !has_fallthrough {
                self.generator.emitter.indent();
                self.generator.emitter.emit("break;\n");
            }

            self.generator.emitter.indent_level -= 1;
        }

        if let Some(else_stmts) = else_block {
            self.generator.emitter.indent();
            self.generator.emitter.emit("default:\n");
            self.generator.emitter.indent_level += 1;

            for stmt in else_stmts {
                self.generate_stmt(stmt);
            }

            self.generator.emitter.indent();
            self.generator.emitter.emit("break;\n");
            self.generator.emitter.indent_level -= 1;
        }

        self.generator.emitter.indent_level -= 1;
        self.generator.emitter.emit_line("}");
    }

    /// Generates C code for a next statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    fn emit_next_stmt(&mut self) {
        self.generator.emitter.emit_line("continue;");
    }

    /// Generates C code for a stop statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    fn emit_stop_stmt(&mut self) {
        self.generator.emitter.emit_line("break;");
    }

    /// Generates C code for a fallthrough statement.
    /// In C, fallthrough is implicit, so we just add a comment.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    fn emit_fallthrough_stmt(&mut self) {
        self.generator.emitter.emit_line("/* fallthrough */");
    }

    /// Emits the step expression for a for loop, or "1" if no step is provided.
    ///
    /// # Parameters
    /// - `step`: Optional step expression
    fn emit_step_value(&mut self, step: &Option<Expression>) {
        if let Some(step_expr) = step {
            let mut expr_gen = ExpressionGenerator::new(self.generator);
            expr_gen.generate_expr(step_expr);
        } else {
            self.generator.emitter.emit("1");
        }
    }

    /// Generates C code for a for loop.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `variable`: The loop variable name
    /// - `start`: The starting expression
    /// - `end`: The ending expression
    /// - `inclusive`: Whether the end value is inclusive
    /// - `step`: Optional step expression
    /// - `filter`: Optional filter expression
    /// - `body`: Loop body statements
    fn emit_for_loop(
        &mut self,
        variable: &str,
        start: &Expression,
        end: &Expression,
        inclusive: bool,
        step: &Option<Expression>,
        filter: &Option<Expression>,
        body: &[Statement],
    ) {
        let type_inference = TypeInference::new(&self.generator.symbol_table,
                                                &self.generator.function_signatures,
                                                &self.generator.struct_defs);
        let start_type = type_inference.infer_expression_type(start);
        let end_type = type_inference.infer_expression_type(end);
        let loop_type = type_inference.wider_type(&start_type, &end_type);
        let c_type = self.generator.map_type(&loop_type).to_string();

        let (_, is_positive) = if let Some(step_expr) = step {
            if let Expression::IntLiteral(val) = step_expr {
                (*val as i64, *val as i64 > 0)
            } else {
                (1, true)
            }
        } else {
            (1, true)
        };

        self.generator.symbol_table.insert(variable.to_string(), loop_type);

        if step.is_none() && filter.is_none() && is_positive {
            self.emit_simple_for_loop(variable, start, end, inclusive, &c_type, body);
        } else {
            self.emit_complex_for_loop(variable, start, end, inclusive, step, filter, body,
                                       &c_type);
        }
    }

    /// Generates a simple for loop (no step, no filter, positive iteration).
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `variable`: The loop variable name
    /// - `start`: The starting expression
    /// - `end`: The ending expression
    /// - `inclusive`: Whether the end value is inclusive
    /// - `c_type`: The C type for the loop variable
    /// - `body`: Loop body statements
    fn emit_simple_for_loop(&mut self, variable: &str, start: &Expression, end: &Expression,
                            inclusive: bool, c_type: &str, body: &[Statement]) {
        self.generator.emitter.indent();
        self.generator.emitter.emit(&format!("for ({} {} = ", c_type, variable));

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(start);

        self.generator.emitter.emit(&format!("; {} {} ", variable,
                                             if inclusive { "<=" } else { "<" }));

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(end);

        self.generator.emitter.emit(&format!("; {}++) {{\n", variable));

        self.generator.emitter.indent_level += 1;

        for stmt in body {
            self.generate_stmt(stmt);
        }

        self.generator.emitter.indent_level -= 1;
        self.generator.emitter.emit_line("}");
    }

    /// Generates a complex for loop.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `variable`: The loop variable name
    /// - `start`: The starting expression
    /// - `end`: The ending expression
    /// - `inclusive`: Whether the end value is inclusive
    /// - `step`: Optional step expression
    /// - `filter`: Optional filter expression
    /// - `body`: Loop body statements
    /// - `c_type`: The C type for the loop variable
    fn emit_complex_for_loop(
        &mut self,
        variable: &str,
        start: &Expression,
        end: &Expression,
        inclusive: bool,
        step: &Option<Expression>,
        filter: &Option<Expression>,
        body: &[Statement],
        c_type: &str,
    ) {
        self.generator.emitter.indent();
        self.generator.emitter.emit("{\n");
        self.generator.emitter.indent_level += 1;

        self.generator.emitter.indent();
        self.generator.emitter.emit(&format!("{} {} = ", c_type, variable));

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(start);

        self.generator.emitter.emit(";\n");

        self.generator.emitter.indent();
        self.generator.emitter.emit(&format!("while ({} ", variable));

        if let Some(step_expr) = step {
            if let Expression::IntLiteral(val) = step_expr {
                if *val as i64 > 0 {
                    self.generator.emitter.emit(if inclusive { "<= " } else { "< " });
                } else {
                    self.generator.emitter.emit(if inclusive { ">= " } else { "> " });
                }
            } else {
                self.generator.emitter.emit(if inclusive { "<= " } else { "< " });
            }
        } else {
            self.generator.emitter.emit(if inclusive { "<= " } else { "< " });
        }

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(end);

        self.generator.emitter.emit(") {\n");

        self.generator.emitter.indent_level += 1;

        if let Some(filter_expr) = filter {
            self.generator.emitter.indent();
            self.generator.emitter.emit("if (!(");

            let mut expr_gen = ExpressionGenerator::new(self.generator);
            expr_gen.generate_expr(filter_expr);

            self.generator.emitter.emit(")) {\n");
            self.generator.emitter.indent_level += 1;
            self.generator.emitter.indent();
            self.generator.emitter.emit(variable);
            self.generator.emitter.emit(" += ");

            self.emit_step_value(step);

            self.generator.emitter.emit(";\n");
            self.generator.emitter.emit_line("continue;");
            self.generator.emitter.indent_level -= 1;
            self.generator.emitter.emit_line("}");
        }

        for stmt in body {
            self.generate_stmt(stmt);
        }

        self.generator.emitter.indent();
        self.generator.emitter.emit(variable);
        self.generator.emitter.emit(" += ");

        self.emit_step_value(step);

        self.generator.emitter.emit(";\n");

        self.generator.emitter.indent_level -= 1;
        self.generator.emitter.emit_line("}");

        self.generator.emitter.indent_level -= 1;
        self.generator.emitter.emit_line("}");
    }

    fn emit_let_stmt(&mut self, name: &str, var_type: &Option<String>, value: &Expression) {
        self.generator.emitter.indent();

        let summit_type = TypeResolver::resolve_type(
            var_type, value, |v| self.generator.infer_expr_type(v));
        let c_type = self.generator.map_type(&summit_type).to_string();

        self.generator.emitter.emit(&c_type);
        self.generator.emitter.emit(" ");
        self.generator.emitter.emit(name);
        self.generator.emitter.emit(" = ");

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(value);

        self.generator.emitter.emit(";\n");

        TypeResolver::register_variable(&mut self.generator.symbol_table, name, summit_type);
    }


    /// Generates C code for a const statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `name`: Variable name
    /// - `var_type`: Optional type annotation
    /// - `value`: Initialization expression
    fn emit_const_stmt(&mut self, name: &str, var_type: &Option<String>, value: &Expression) {
        self.generator.emitter.indent();

        let summit_type = TypeResolver::resolve_type(
            var_type, value, |v| self.generator.infer_expr_type(v));
        let c_type = self.generator.map_type(&summit_type).to_string();

        self.generator.emitter.emit("const ");
        self.generator.emitter.emit(&c_type);
        self.generator.emitter.emit(" ");
        self.generator.emitter.emit(name);
        self.generator.emitter.emit(" = ");

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(value);

        self.generator.emitter.emit(";\n");

        TypeResolver::register_variable(&mut self.generator.symbol_table, name, summit_type);
    }

    /// Generates C code for a comptime statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `name`: Variable name
    /// - `var_type`: Optional type annotation
    /// - `value`: Initialization expression
    fn emit_comptime_stmt(&mut self, name: &str, var_type: &Option<String>,
                          value: &Expression) {
        self.emit_const_stmt(name, var_type, value);
    }

    /// Generates C code for an assignment statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `name`: Variable name
    /// - `value`: Assignment expression
    fn emit_assign_stmt(&mut self, name: &str, value: &Expression) {
        self.generator.emitter.indent();
        self.generator.emitter.emit(name);
        self.generator.emitter.emit(" = ");

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(value);

        self.generator.emitter.emit(";\n");
    }

    /// Generates C code for a field assignment statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `object`: The object name whose field is being assigned
    /// - `field`: The field name being assigned
    /// - `value`: Assignment expression
    fn emit_field_assign_stmt(&mut self, object: &str, field: &str, value: &Expression) {
        self.generator.emitter.indent();
        self.generator.emitter.emit(object);
        self.generator.emitter.emit(".");
        self.generator.emitter.emit(field);
        self.generator.emitter.emit(" = ");

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(value);

        self.generator.emitter.emit(";\n");
    }

    /// Generates C code for a return statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `expr`: Return expression
    fn emit_ret_stmt(&mut self, expr: &Expression) {
        self.generator.emitter.indent();
        self.generator.emitter.emit("return ");

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(expr);

        self.generator.emitter.emit(";\n");
    }

    /// Generates C code for an expression statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `expr`: The expression
    fn emit_expr_stmt(&mut self, expr: &Expression) {
        self.generator.emitter.indent();

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(expr);

        self.generator.emitter.emit(";\n");
    }

    /// Generates C code for an if statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `condition`: The if condition
    /// - `then_block`: Statements in the then block
    /// - `elseif_blocks`: Optional elseif blocks
    /// - `else_block`: Optional statements in the else block
    fn emit_if_stmt(&mut self, condition: &Expression, then_block: &[Statement],
                    elseif_blocks: &[ElseIfBlock], else_block: &Option<Vec<Statement>>) {
        self.generator.emitter.indent();
        self.generator.emitter.emit("if (");

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(condition);

        self.generator.emitter.emit(") {\n");

        self.generator.emitter.indent_level += 1;
        for stmt in then_block {
            self.generate_stmt(stmt);
        }
        self.generator.emitter.indent_level -= 1;

        for elseif_block in elseif_blocks {
            self.generator.emitter.emit_line("} else if (");

            let mut expr_gen = ExpressionGenerator::new(self.generator);
            expr_gen.generate_expr(&elseif_block.condition);

            self.generator.emitter.emit(") {\n");
            self.generator.emitter.indent_level += 1;

            for stmt in &elseif_block.body {
                self.generate_stmt(stmt);
            }

            self.generator.emitter.indent_level -= 1;
        }

        if let Some(else_stmts) = else_block {
            self.generator.emitter.emit_line("} else {");
            self.generator.emitter.indent_level += 1;
            for stmt in else_stmts {
                self.generate_stmt(stmt);
            }
            self.generator.emitter.indent_level -= 1;
        }

        self.generator.emitter.emit_line("}");
    }

    /// Generates C code for a while statement.
    ///
    /// # Parameters
    /// - `self`: Mutable reference to self
    /// - `condition`: The while condition
    /// - `body`: Statements in the loop body
    fn emit_while_stmt(&mut self, condition: &Expression, body: &[Statement]) {
        self.generator.emitter.indent();
        self.generator.emitter.emit("while (");

        let mut expr_gen = ExpressionGenerator::new(self.generator);
        expr_gen.generate_expr(condition);

        self.generator.emitter.emit(") {\n");

        self.generator.emitter.indent_level += 1;
        for stmt in body {
            self.generate_stmt(stmt);
        }
        self.generator.emitter.indent_level -= 1;

        self.generator.emitter.emit_line("}");
    }
}