//! Benchmark tests for symbols extraction performance.
//!
//! These tests measure the performance of symbol extraction from Rust and Python
//! source files. They are designed to be run with `cargo test` and validate
//! that symbol extraction completes within acceptable time limits.

use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::PathBuf;
use tempfile::NamedTempFile;

use xiuxian_wendao::SymbolIndex;
use xiuxian_wendao::dependency_indexer::{ExternalSymbol, SymbolKind, extract_symbols};

fn append_format(content: &mut String, args: std::fmt::Arguments<'_>) {
    if content.write_fmt(args).is_err() {
        unreachable!("formatting into String should not fail");
    }
}

/// Generate a large Rust source file for benchmarking.
fn generate_rust_test_file(line_count: usize) -> String {
    let mut content = String::with_capacity(line_count * 50);

    // Add structs
    for i in 0..(line_count / 50) {
        append_format(
            &mut content,
            format_args!(
                "pub struct Struct{i} {{\n    field_{i}: String,\n    field_{i}: i32,\n}}\n"
            ),
        );
    }

    // Add enums
    for i in 0..(line_count / 100) {
        append_format(
            &mut content,
            format_args!(
                "pub enum Enum{i} {{\n    VariantA,\n    VariantB(i32),\n    VariantC {{ x: i32, y: i32 }},\n}}\n"
            ),
        );
    }

    // Add functions
    for i in 0..(line_count / 30) {
        append_format(
            &mut content,
            format_args!(
                "pub fn function_{i}(arg1: &str, arg2: i32) -> Result<(), Box<dyn std::error::Error>> {{\n    let _result = process_data(arg1, arg2);\n    Ok(())\n}}\n"
            ),
        );
    }

    // Add traits
    for i in 0..(line_count / 80) {
        append_format(
            &mut content,
            format_args!(
                "pub trait Trait{i} {{\n    fn method_a(&self) -> i32;\n    fn method_b(&self, x: i32) -> bool;\n}}\n"
            ),
        );
    }

    content
}

/// Generate a large Python source file for benchmarking.
fn generate_python_test_file(line_count: usize) -> String {
    let mut content = String::with_capacity(line_count * 40);

    // Add classes
    for i in 0..(line_count / 50) {
        append_format(
            &mut content,
            format_args!(
                "class Class{i}:\n    def __init__(self, param_a: str, param_b: int):\n        self.param_a = param_a\n        self.param_b = param_b\n\n    def method_a(self) -> str:\n        return self.param_a.upper()\n\n    def method_b(self, value: int) -> bool:\n        return value > 0\n\n    async def async_method(self) -> dict:\n        return {{\"status\": \"ok\"}}\n"
            ),
        );
    }

    // Add functions
    for i in 0..(line_count / 20) {
        append_format(
            &mut content,
            format_args!(
                "def function_{i}(arg1: str, arg2: int) -> bool:\n    \"\"\"Process data and return result.\"\"\"\n    result = process(arg1, arg2)\n    return result\n\nasync def async_function_{i}(data: dict) -> list:\n    \"\"\"Async data processing.\"\"\"\n    results = []\n    return results\n"
            ),
        );
    }

    content
}

mod mixed_symbol_extraction_performance;
mod python_symbol_extraction_performance;
/// Benchmark test for Rust symbol extraction.
mod rust_symbol_extraction_performance;
mod symbol_index_search_performance;
