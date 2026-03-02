//! Extract symbols from Rust/Python source files using omni-tags.

mod extract;
mod index;
mod model;

pub use extract::extract_symbols;
pub use index::SymbolIndex;
pub use model::{ExternalSymbol, SymbolKind};
