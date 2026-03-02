//! `PyO3` bindings for `UnifiedSymbolIndex`.
//!
//! This module provides Python bindings for searching across both project symbols
//! and external dependency symbols in a unified way.

mod py_index;
mod py_stats;
mod py_symbol;
mod registration;

pub use py_index::PyUnifiedSymbolIndex;
pub use py_stats::PyUnifiedIndexStats;
pub use py_symbol::PyUnifiedSymbol;
pub use registration::register_unified_symbol_module;
