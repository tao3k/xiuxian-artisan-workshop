/// Manifestation manager logic.
pub mod manager;
/// Template helper logic.
pub mod templates;

pub use manager::ManifestationManager;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::ManifestationInterface;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_template_rendering() {
        let dir = tempdir().unwrap();
        let template_path = dir.path().join("test.md.j2");
        fs::write(&template_path, "Hello {{ name }}!").unwrap();
        
        // Tera needs a glob pattern
        let glob = format!("{}/*.j2", dir.path().to_str().unwrap());
        let manager = ManifestationManager::new(&glob).unwrap();
        
        let result = manager.render_template("test.md.j2", json!({"name": "Daoist"})).unwrap();
        assert_eq!(result, "Hello Daoist!");
    }
}
