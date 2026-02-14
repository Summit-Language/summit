use crate::frontend::ast::*;
use std::collections::HashMap;
use super::analyzer::SemanticAnalyzer;
use super::module_checker::ModuleChecker;
use crate::utils::io_path_matcher::IoPathMatcher;

/// Analyzes expressions for semantic correctness.
pub struct ExpressionAnalyzer<'a> {
    /// The main program analyzer checking for semantic correctness
    analyzer: &'a SemanticAnalyzer,
}

impl<'a> ExpressionAnalyzer<'a> {
    /// Creates a new expression analyzer.
    ///
    /// # Parameters
    /// - `analyzer`: Reference to the semantic analyzer
    ///
    /// # Returns
    /// A new ExpressionAnalyzer instance
    pub fn new(analyzer: &'a SemanticAnalyzer) -> Self {
        ExpressionAnalyzer { analyzer }
    }

    /// Analyzes an expression for semantic correctness.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `expr`: The expression to analyze
    /// - `scope`: Current variable scope mapping names to types
    ///
    /// # Returns
    /// Ok(()) if analysis succeeds, Err with message on failure
    pub fn analyze_expr(&self, expr: &Expression,
                        scope: &HashMap<String, String>) -> Result<(), String> {
        match expr {
            Expression::IntLiteral(_) | Expression::StringLiteral(_) | Expression::BoolLiteral(_)
            | Expression::NullLiteral => Ok(()),
            Expression::Variable(name) => {
                if !scope.contains_key(name) {
                    Err(format!("Undefined variable: {}", name))
                } else {
                    Ok(())
                }
            }
            Expression::Call { path, type_args,
                args } => {
                let module_checker = ModuleChecker::new(self.analyzer);

                if IoPathMatcher::is_read(path) {
                    if type_args.is_none() {
                        return Err("io::read requires a type parameter, e.g., io::read<i32>()"
                            .to_string());
                    }

                    if let Some(types) = type_args {
                        if types.len() != 1 {
                            return Err("io::read requires exactly one type parameter".to_string());
                        }

                        let type_name = &types[0];
                        if !module_checker.is_valid_read_type(type_name) {
                            return Err(format!("io::read does not support type '{}'. Supported types: i8, u8, i16, u16, i32, u32, i64, u64", type_name));
                        }
                    }

                    if !args.is_empty() {
                        return Err("io::read takes no arguments".to_string());
                    }

                    return Ok(());
                }

                if IoPathMatcher::is_readln(path) {
                    if type_args.is_some() {
                        return Err("io::readln does not take type parameters".to_string());
                    }

                    if !args.is_empty() {
                        return Err("io::readln takes no arguments".to_string());
                    }

                    return Ok(());
                }

                if path.len() >= 2 {
                    module_checker.check_module_import(path)?;
                } else if path.len() == 1 {
                    self.check_local_func_call(path, args, scope)?;
                }

                for arg in args {
                    self.analyze_expr(arg, scope)?;
                }
                Ok(())
            }
            Expression::Binary { left, right, .. } => {
                self.analyze_expr(left, scope)?;
                self.analyze_expr(right, scope)?;

                let left_type = self.analyzer.type_checker.infer_type(left, scope,
                                                                      &self.analyzer.functions,
                                                                      &self.analyzer.structs)?;
                let right_type = self.analyzer.type_checker.infer_type(right, scope,
                                                                       &self.analyzer.functions,
                                                                       &self.analyzer.structs)?;

                if !self.analyzer.type_checker.types_compatible(&left_type, &right_type) {
                    return Err(format!("Type mismatch in binary operation: {} and {}",
                                       left_type, right_type));
                }

                Ok(())
            }
            Expression::Unary { operand, .. } => {
                self.analyze_expr(operand, scope)
            }
            Expression::IfExpr { condition, then_expr,
                else_expr } => {
                self.analyze_expr(condition, scope)?;

                self.analyze_expr(then_expr, scope)?;
                self.analyze_expr(else_expr, scope)?;

                let then_type = self.analyzer.type_checker.infer_type(then_expr, scope,
                                                                      &self.analyzer.functions,
                                                                      &self.analyzer.structs)?;
                let else_type = self.analyzer.type_checker.infer_type(else_expr, scope,
                                                                      &self.analyzer.functions,
                                                                      &self.analyzer.structs)?;

                if !self.analyzer.type_checker.types_compatible(&then_type, &else_type) {
                    return Err(format!(
                        "If expression branches have incompatible types: '{}' and '{}'",
                        then_type, else_type
                    ));
                }

                Ok(())
            }
            Expression::WhenExpr { value, cases, else_expr } => {
                self.analyze_expr(value, scope)?;

                let value_type = self.analyzer.type_checker.infer_type(value, scope,
                                                                       &self.analyzer.functions,
                                                                       &self.analyzer.structs)?;

                let mut result_types = Vec::new();

                for case in cases {
                    let mut case_scope = scope.clone();

                    match &case.pattern {
                        WhenPattern::Single(pattern_expr) => {
                            self.analyze_expr(pattern_expr, scope)?;

                            let pattern_type = self.analyzer.type_checker.infer_type(
                                pattern_expr, scope, &self.analyzer.functions, &self.analyzer.structs)?;

                            if !self.analyzer.type_checker.types_compatible(&value_type, &pattern_type) {
                                return Err(format!(
                                    "When pattern type '{}' is incompatible with value type '{}'",
                                    pattern_type, value_type
                                ));
                            }
                        }
                        WhenPattern::Range { start, end, .. } => {
                            self.analyze_expr(start, scope)?;
                            self.analyze_expr(end, scope)?;

                            let start_type = self.analyzer.type_checker.infer_type(
                                start, scope, &self.analyzer.functions, &self.analyzer.structs)?;
                            let end_type = self.analyzer.type_checker.infer_type(
                                end, scope, &self.analyzer.functions, &self.analyzer.structs)?;

                            if !self.analyzer.type_checker.types_compatible(&value_type, &start_type) {
                                return Err(format!(
                                    "When range start type '{}' is incompatible with value type '{}'",
                                    start_type, value_type
                                ));
                            }

                            if !self.analyzer.type_checker.types_compatible(&value_type, &end_type) {
                                return Err(format!(
                                    "When range end type '{}' is incompatible with value type '{}'",
                                    end_type, value_type
                                ));
                            }
                        }
                        WhenPattern::EnumVariant { enum_name, variant_name, bindings } => {
                            if value_type != *enum_name {
                                return Err(format!(
                                    "When pattern '{}::{}' expects enum type '{}', but value has type '{}'",
                                    enum_name, variant_name, enum_name, value_type
                                ));
                            }

                            if let Some(enum_def) = self.analyzer.enums.get(enum_name) {
                                if let Some(variant) = enum_def.variants.iter().find(|v| &v.name == variant_name) {
                                    let payload_count = variant.payload.as_ref().map(|p| p.len()).unwrap_or(0);
                                    if bindings.len() != payload_count {
                                        return Err(format!(
                                            "Enum variant '{}::{}' expects {} bindings, but {} were provided",
                                            enum_name, variant_name, payload_count, bindings.len()
                                        ));
                                    }

                                    if let Some(payload_types) = &variant.payload {
                                        for (binding_name, payload_type) in bindings.iter().zip(payload_types.iter()) {
                                            case_scope.insert(binding_name.clone(), payload_type.clone());
                                        }
                                    }
                                } else {
                                    return Err(format!(
                                        "Enum variant '{}::{}' not found in enum '{}'",
                                        enum_name, variant_name, enum_name
                                    ));
                                }
                            } else {
                                for binding_name in bindings {
                                    case_scope.insert(binding_name.clone(), "i32".to_string());
                                }
                            }
                        }
                    }

                    self.analyze_expr(&case.result, &case_scope)?;

                    let result_type = self.analyzer.type_checker.infer_type(
                        &case.result, &case_scope, &self.analyzer.functions, &self.analyzer.structs)?;
                    result_types.push(result_type);
                }

                self.analyze_expr(else_expr, scope)?;
                let else_type = self.analyzer.type_checker.infer_type(
                    else_expr, scope, &self.analyzer.functions, &self.analyzer.structs)?;
                result_types.push(else_type);

                for i in 1..result_types.len() {
                    if !self.analyzer.type_checker.types_compatible(&result_types[0], &result_types[i]) {
                        return Err(format!(
                            "When expression branches have incompatible result types: '{}' and '{}'",
                            result_types[0], result_types[i]
                        ));
                    }
                }

                Ok(())
            }
            Expression::StructInit { struct_name, fields } => {
                if !self.analyzer.structs.contains_key(struct_name) {
                    return Err(format!("Undefined struct: {}", struct_name));
                }

                let struct_def = &self.analyzer.structs[struct_name];

                if fields.is_empty() {
                    return Err(format!("Struct '{}' must be initialized with fields", struct_name));
                }

                let has_named = fields.iter().any(|f| f.name.is_some());
                let has_positional = fields.iter().any(|f| f.name.is_none());

                if has_named && has_positional {
                    return Err(format!("Struct '{}' cannot mix named and positional field initialization", struct_name));
                }

                if has_positional {
                    if fields.len() != struct_def.fields.len() {
                        return Err(format!(
                            "Struct '{}' expects {} fields, but {} were provided",
                            struct_name, struct_def.fields.len(), fields.len()
                        ));
                    }

                    for (i, (field_init, struct_field)) in fields.iter().zip(struct_def.fields.iter()).enumerate() {
                        self.analyze_expr(&field_init.value, scope)?;
                        let value_type = self.analyzer.type_checker.infer_type(
                            &field_init.value, scope, &self.analyzer.functions, &self.analyzer.structs)?;

                        if !self.analyzer.type_checker.types_compatible(&struct_field.field_type, &value_type) {
                            return Err(format!(
                                "Type mismatch in field {} of struct '{}': expected '{}', got '{}'",
                                i + 1, struct_name, struct_field.field_type, value_type
                            ));
                        }
                    }
                } else {
                    let mut initialized_fields = HashMap::new();

                    for field_init in fields {
                        if let Some(field_name) = &field_init.name {
                            if initialized_fields.contains_key(field_name) {
                                return Err(format!(
                                    "Field '{}' is initialized multiple times in struct '{}'",
                                    field_name, struct_name
                                ));
                            }

                            let struct_field = struct_def.fields.iter()
                                .find(|f| &f.name == field_name)
                                .ok_or_else(|| format!(
                                    "Field '{}' not found in struct '{}'",
                                    field_name, struct_name
                                ))?;

                            self.analyze_expr(&field_init.value, scope)?;
                            let value_type = self.analyzer.type_checker.infer_type(
                                &field_init.value, scope, &self.analyzer.functions, &self.analyzer.structs)?;

                            if !self.analyzer.type_checker.types_compatible(&struct_field.field_type, &value_type) {
                                return Err(format!(
                                    "Type mismatch in field '{}' of struct '{}': expected '{}', got '{}'",
                                    field_name, struct_name, struct_field.field_type, value_type
                                ));
                            }

                            initialized_fields.insert(field_name.clone(), ());
                        }
                    }

                    for struct_field in &struct_def.fields {
                        if !initialized_fields.contains_key(&struct_field.name) {
                            return Err(format!(
                                "Field '{}' is not initialized in struct '{}'",
                                struct_field.name, struct_name
                            ));
                        }
                    }
                }

                Ok(())
            }
            Expression::FieldAccess { object, field } => {
                self.analyze_expr(object, scope)?;

                let object_type = self.analyzer.type_checker.infer_type(
                    object, scope, &self.analyzer.functions, &self.analyzer.structs)?;

                if !self.analyzer.structs.contains_key(&object_type) {
                    return Err(format!("Cannot access field '{}' on non-struct type '{}'", field, object_type));
                }

                let struct_def = &self.analyzer.structs[&object_type];
                let field_exists = struct_def.fields.iter().any(|f| &f.name == field);

                if !field_exists {
                    return Err(format!("Field '{}' not found in struct '{}'", field, object_type));
                }

                Ok(())
            }
            Expression::EnumConstruct { enum_name, variant_name, args } => {
                if !self.analyzer.enums.contains_key(enum_name) {
                    return Err(format!("Undefined enum: {}", enum_name));
                }

                let enum_def = &self.analyzer.enums[enum_name];

                let variant = enum_def.variants.iter()
                    .find(|v| &v.name == variant_name)
                    .ok_or_else(|| format!(
                        "Variant '{}' not found in enum '{}'",
                        variant_name, enum_name
                    ))?;

                let expected_arg_count = variant.payload.as_ref().map(|p| p.len()).unwrap_or(0);

                if args.len() != expected_arg_count {
                    return Err(format!(
                        "Enum variant '{}::{}' expects {} arguments, but {} were provided",
                        enum_name, variant_name, expected_arg_count, args.len()
                    ));
                }

                if let Some(payload_types) = &variant.payload {
                    for (i, (arg, expected_type)) in args.iter().zip(payload_types.iter()).enumerate() {
                        self.analyze_expr(arg, scope)?;

                        let arg_type = self.analyzer.type_checker.infer_type(
                            arg, scope, &self.analyzer.functions, &self.analyzer.structs)?;

                        if !self.analyzer.type_checker.types_compatible(&arg_type, expected_type) {
                            return Err(format!(
                                "Type mismatch in argument {} of enum variant '{}::{}': expected '{}', got '{}'",
                                i + 1, enum_name, variant_name, expected_type, arg_type
                            ));
                        }
                    }
                } else {
                    for arg in args {
                        self.analyze_expr(arg, scope)?;
                    }
                }

                Ok(())
            }
        }
    }

    /// Validates a local function call.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `path`: Function path
    /// - `args`: Function arguments
    /// - `scope`: Current variable scope
    ///
    /// # Returns
    /// Ok(()) if the call is valid, Err with message on failure
    fn check_local_func_call(&self, path: &[String], args: &[Expression],
                             scope: &HashMap<String, String>) -> Result<(), String> {
        let func_name = &path[0];

        if !self.analyzer.functions.contains_key(func_name) {
            return Err(format!("Undefined function: '{}'", func_name));
        }

        if let Some(params) = self.analyzer.function_params.get(func_name) {
            let func = &self.analyzer.functions[func_name];
            let is_variadic = func.has_varargs;

            if is_variadic {
                if args.len() < params.len() {
                    return Err(format!(
                        "Function '{}' expects at least {} arguments, but {} were provided",
                        func_name, params.len(), args.len()
                    ));
                }
            } else {
                if args.len() != params.len() {
                    return Err(format!(
                        "Function '{}' expects {} arguments, but {} were provided",
                        func_name, params.len(), args.len()
                    ));
                }
            }

            for (i, (arg, (param_name, param_type))) in args
                .iter().zip(params.iter()).enumerate() {
                let arg_type = self.analyzer.type_checker.infer_type(arg, scope,
                                                                     &self.analyzer.functions,
                                                                     &self.analyzer.structs)?;
                if arg_type == *param_type {
                    continue;
                }

                if self.analyzer.type_checker.can_convert(&arg_type, param_type) {
                    continue;
                }

                if self.analyzer.type_checker.is_signed(&arg_type)
                    && !self.analyzer.type_checker.is_signed(param_type) {
                    let arg_size = self.analyzer.type_checker.get_type_size(&arg_type);
                    let param_size = self.analyzer.type_checker.get_type_size(param_type);

                    if arg_size < param_size {
                        if let Some(val) = self.analyzer.type_checker.get_literal_value(arg) {
                            if val > self.analyzer.type_checker.get_unsigned_max(param_type) {
                                return Err(format!(
                                    "Cannot convert value {} to unsigned type '{}' (exceeds maximum: {})",
                                    val, param_type, self.analyzer.type_checker
                                        .get_unsigned_max(param_type)
                                ));
                            }
                        }
                        continue;
                    }
                }

                if !self.analyzer.type_checker.is_signed(&arg_type)
                    && self.analyzer.type_checker.is_signed(param_type) {
                    let arg_size = self.analyzer.type_checker.get_type_size(&arg_type);
                    let param_size = self.analyzer.type_checker.get_type_size(param_type);

                    if arg_size < param_size {
                        continue;
                    }
                }

                if self.analyzer.type_checker.would_truncate(&arg_type, param_type) {
                    let error_msg = match arg {
                        Expression::Variable(var_name) => {
                            format!(
                                "Type mismatch in argument {} of function '{}': variable '{}' has inferred type '{}', but parameter '{}' expects type '{}'. The value assigned to '{}' requires type '{}', which exceeds the range of '{}'",
                                i + 1, func_name, var_name, arg_type, param_name,
                                param_type, var_name, arg_type, param_type
                            )
                        }
                        Expression::IntLiteral(val) => {
                            format!(
                                "Type mismatch in argument {} of function '{}': literal value {} exceeds maximum value for parameter '{}' of type '{}' (maximum: {})",
                                i + 1, func_name, val, param_name, param_type,
                                self.analyzer.type_checker.get_type_max(param_type)
                            )
                        }
                        _ => {
                            format!(
                                "Type mismatch in argument {} of function '{}': expression has type '{}' but parameter '{}' expects type '{}' (would lose data)",
                                i + 1, func_name, arg_type, param_name, param_type
                            )
                        }
                    };
                    return Err(error_msg);
                }

                if !self.analyzer.type_checker.types_compatible(&arg_type, param_type) {
                    return Err(format!(
                        "Type mismatch in argument {} of function '{}': expected '{}', got '{}'",
                        i + 1, func_name, param_type, arg_type
                    ));
                }

                return Err(format!(
                    "Type mismatch in argument {} of function '{}': cannot implicitly convert '{}' to '{}' (same size but different signedness)",
                    i + 1, func_name, arg_type, param_type
                ));
            }
        }

        Ok(())
    }
}