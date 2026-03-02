use super::loader::{collect_persona_files, parse_profile_from_file};
use super::profile::PersonaProfile;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// In-memory persona payload resolved from runtime indexes (for example Wendao).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryPersonaRecord {
    /// Exact persona identifier used for lookup.
    pub id: String,
    /// Raw TOML profile payload.
    pub content: String,
}

impl MemoryPersonaRecord {
    /// Creates an in-memory persona record.
    #[must_use]
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
        }
    }
}

/// Read-through provider for persona profiles on cache misses.
pub trait PersonaProvider: Send + Sync {
    /// Fetches one persona profile by exact identifier.
    fn fetch_persona(&self, id: &str) -> Option<PersonaProfile>;
}

/// Registry managing the collection of available personas.
pub struct PersonaRegistry {
    personas: RwLock<HashMap<String, PersonaProfile>>,
    provider: Option<Arc<dyn PersonaProvider>>,
}

impl std::fmt::Debug for PersonaRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let personas_len = self.read_personas().len();
        f.debug_struct("PersonaRegistry")
            .field("personas_len", &personas_len)
            .field("provider_enabled", &self.provider.is_some())
            .finish_non_exhaustive()
    }
}

impl Default for PersonaRegistry {
    fn default() -> Self {
        Self {
            personas: RwLock::new(HashMap::new()),
            provider: None,
        }
    }
}

impl PersonaRegistry {
    /// Creates an empty persona registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty registry with a cache-miss provider.
    #[must_use]
    pub fn with_provider(provider: Arc<dyn PersonaProvider>) -> Self {
        Self {
            personas: RwLock::new(HashMap::new()),
            provider: Some(provider),
        }
    }

    /// Sets or replaces the cache-miss provider.
    pub fn set_provider(&mut self, provider: Arc<dyn PersonaProvider>) {
        self.provider = Some(provider);
    }

    /// Creates a new registry with built-in personas loaded from runtime directories.
    ///
    /// Built-in resources are loaded from this crate under:
    /// `resources/qianhuan/personas`.
    ///
    /// User personas are loaded separately by host runtime bootstrap.
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut dirs = vec![
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join("qianhuan")
                .join("personas"),
        ];
        // Backward-compat seed path during migration.
        let legacy_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("personas");
        if legacy_path.is_dir() {
            dirs.push(legacy_path);
        }

        match Self::load_from_dirs(&dirs) {
            Ok(registry) => registry,
            Err(error) => {
                log::warn!(
                    "failed to load builtin personas from runtime directories; using empty registry: {error}"
                );
                Self::default()
            }
        }
    }

    /// Loads persona profiles from a single directory recursively.
    ///
    /// Supported file format is `.toml`.
    ///
    /// # Errors
    ///
    /// Returns an error when the directory exists but cannot be traversed,
    /// or when any discovered profile file fails to parse.
    pub fn load_from_dir(path: &Path) -> Result<Self> {
        let registry = Self::new();
        for file_path in collect_persona_files(path)? {
            let profile = parse_profile_from_file(&file_path)?;
            registry.register(profile);
        }
        Ok(registry)
    }

    /// Loads persona profiles from multiple directories.
    ///
    /// Later directories override earlier ones for duplicate persona IDs.
    ///
    /// # Errors
    ///
    /// Returns an error when any directory traversal or file parsing fails.
    pub fn load_from_dirs(paths: &[PathBuf]) -> Result<Self> {
        let registry = Self::new();
        for path in paths {
            for file_path in collect_persona_files(path)? {
                let profile = parse_profile_from_file(&file_path)?;
                registry.register(profile);
            }
        }
        Ok(registry)
    }

    /// Fetches a persona profile by its unique ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<PersonaProfile> {
        if let Some(profile) = self.read_personas().get(id).cloned() {
            return Some(profile);
        }
        let provider = self.provider.as_ref()?;
        let mut profile = provider.fetch_persona(id)?;
        if profile.id.trim() != id {
            profile.id = id.to_string();
        }
        self.write_personas()
            .insert(id.to_string(), profile.clone());
        Some(profile)
    }

    /// Returns the total number of registered personas.
    #[must_use]
    pub fn len(&self) -> usize {
        self.read_personas().len()
    }

    /// Returns `true` when no persona profiles are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.read_personas().is_empty()
    }

    /// Registers a custom persona into the registry.
    pub fn register(&self, profile: PersonaProfile) {
        self.write_personas().insert(profile.id.clone(), profile);
    }

    /// Registers one persona profile from runtime TOML payload.
    ///
    /// The provided `id` is enforced as the registry key to guarantee exact
    /// lookups from external memory indexes.
    ///
    /// # Errors
    ///
    /// Returns an error when TOML parsing fails or required fields are missing.
    pub fn register_from_memory_toml(&self, id: &str, persona_toml: &str) -> Result<()> {
        let mut payload: toml::Value = toml::from_str(persona_toml)
            .map_err(|error| anyhow!("failed to parse persona TOML for '{id}': {error}"))?;
        let table = payload
            .as_table_mut()
            .ok_or_else(|| anyhow!("persona payload for '{id}' must be a TOML table"))?;

        table.insert("id".to_string(), toml::Value::String(id.to_string()));

        let profile: PersonaProfile = payload.try_into().map_err(|error| {
            anyhow!("failed to deserialize persona profile for '{id}': {error}")
        })?;
        self.register(profile);
        Ok(())
    }

    /// Bulk-loads persona profiles from runtime memory records.
    ///
    /// # Errors
    ///
    /// Returns an error when any profile payload fails to parse.
    pub fn load_from_memory_records<I>(&mut self, records: I) -> Result<usize>
    where
        I: IntoIterator<Item = MemoryPersonaRecord>,
    {
        let mut loaded = 0usize;
        for record in records {
            self.register_from_memory_toml(&record.id, &record.content)?;
            loaded += 1;
        }
        Ok(loaded)
    }

    fn read_personas(&self) -> RwLockReadGuard<'_, HashMap<String, PersonaProfile>> {
        match self.personas.read() {
            Ok(guard) => guard,
            Err(error) => {
                log::warn!(
                    "persona registry read lock poisoned; recovering poisoned state: {error}"
                );
                error.into_inner()
            }
        }
    }

    fn write_personas(&self) -> RwLockWriteGuard<'_, HashMap<String, PersonaProfile>> {
        match self.personas.write() {
            Ok(guard) => guard,
            Err(error) => {
                log::warn!(
                    "persona registry write lock poisoned; recovering poisoned state: {error}"
                );
                error.into_inner()
            }
        }
    }
}
