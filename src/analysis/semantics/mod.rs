mod analyzer;
mod expression_analyzer;
mod statement_analyzer;
mod function_analyzer;
mod scope_manager;
mod mutation_checker;
mod compile_time_checker;
mod module_checker;

pub use analyzer::analyze;
