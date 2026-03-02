//! Integration tests for dependency symbol extraction.

use std::io::Write as IoWrite;
use std::path::PathBuf;

use xiuxian_wendao::dependency_indexer::{
    ExternalSymbol, SymbolIndex, SymbolKind, extract_symbols,
};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_extract_rust_symbols() -> TestResult {
    let temp_file = tempfile::NamedTempFile::new()?;
    {
        let mut f = std::io::BufWriter::new(&temp_file);
        writeln!(f, "pub struct MyStruct {{")?;
        writeln!(f, "    field: String,")?;
        writeln!(f, "}}")?;
        writeln!(f)?;
        writeln!(f, "pub enum MyEnum {{")?;
        writeln!(f, "    Variant,")?;
        writeln!(f, "}}")?;
        writeln!(f)?;
        writeln!(f, "pub fn my_function() {{")?;
        writeln!(f, "}}")?;
    }

    let symbols = extract_symbols(temp_file.path(), "rust")?;

    assert!(
        symbols
            .iter()
            .any(|s| s.name == "MyStruct" && s.kind == SymbolKind::Struct)
    );
    assert!(
        symbols
            .iter()
            .any(|s| s.name == "MyEnum" && s.kind == SymbolKind::Enum)
    );
    assert!(
        symbols
            .iter()
            .any(|s| s.name == "my_function" && s.kind == SymbolKind::Function)
    );
    Ok(())
}

#[test]
fn test_extract_python_symbols() -> TestResult {
    let temp_file = tempfile::NamedTempFile::new()?;
    {
        let mut f = std::io::BufWriter::new(&temp_file);
        writeln!(f, "class MyClass:")?;
        writeln!(f, "    pass")?;
        writeln!(f)?;
        writeln!(f, "def my_function():")?;
        writeln!(f, "    pass")?;
    }

    let symbols = extract_symbols(temp_file.path(), "python")?;

    assert!(
        symbols
            .iter()
            .any(|s| s.name == "MyClass" && s.kind == SymbolKind::Struct)
    );
    assert!(
        symbols
            .iter()
            .any(|s| s.name == "my_function" && s.kind == SymbolKind::Function)
    );
    Ok(())
}

#[test]
fn test_symbol_index_search() {
    let mut index = SymbolIndex::new();

    index.add_symbols(
        "serde",
        &[
            ExternalSymbol {
                name: "Serializer".to_string(),
                kind: SymbolKind::Struct,
                file: PathBuf::from("lib.rs"),
                line: 10,
                crate_name: "serde".to_string(),
            },
            ExternalSymbol {
                name: "serialize".to_string(),
                kind: SymbolKind::Function,
                file: PathBuf::from("lib.rs"),
                line: 20,
                crate_name: "serde".to_string(),
            },
        ],
    );

    index.add_symbols(
        "tokio",
        &[ExternalSymbol {
            name: "spawn".to_string(),
            kind: SymbolKind::Function,
            file: PathBuf::from("lib.rs"),
            line: 5,
            crate_name: "tokio".to_string(),
        }],
    );

    let results = index.search("serialize", 10);
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|s| s.name == "Serializer"));
    assert!(results.iter().any(|s| s.name == "serialize"));

    let results = index.search("spawn", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "spawn");

    let results = index.search_crate("serde", "serialize", 10);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_serialize_deserialize() {
    let mut index = SymbolIndex::new();

    index.add_symbols(
        "test",
        &[ExternalSymbol {
            name: "MyStruct".to_string(),
            kind: SymbolKind::Struct,
            file: PathBuf::from("lib.rs"),
            line: 10,
            crate_name: "test".to_string(),
        }],
    );

    let data = index.serialize();

    let mut index2 = SymbolIndex::new();
    let _ = index2.deserialize(&data);

    let results = index2.search("MyStruct", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "MyStruct");
}
