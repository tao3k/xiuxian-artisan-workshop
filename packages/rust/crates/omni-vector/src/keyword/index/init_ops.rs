impl KeywordIndex {
    /// Helper to create a fresh index with correct schema.
    fn create_new_index(path: &Path) -> Result<Index, TantivyError> {
        use tantivy::schema::TextFieldIndexing;

        let mut schema_builder = Schema::builder();

        // Use `code_tokenizer` with FULL indexing (including positions for phrase queries).
        let text_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("code_tokenizer")
                    .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        schema_builder.add_text_field("tool_name", text_options.clone());
        schema_builder.add_text_field("description", text_options.clone());
        schema_builder.add_text_field("category", text_options.clone());
        schema_builder.add_text_field("keywords", text_options.clone());
        schema_builder.add_text_field("intents", text_options);

        let schema = schema_builder.build();
        Index::create_in_dir(path, schema)
    }

    /// Create a new `KeywordIndex` with schema migration (deletes old index if needed).
    fn new_with_migration<P: AsRef<Path>>(path: P) -> Result<Self, VectorStoreError> {
        let base_path = path.as_ref();
        let index_path = base_path.join("keyword_index");

        // Remove old index directory if it exists.
        if index_path.exists() {
            std::fs::remove_dir_all(&index_path).map_err(|e| {
                VectorStoreError::General(format!("Failed to remove old index: {e}"))
            })?;
        }

        // Create fresh index with correct schema.
        let index = Self::create_new_index(&index_path).map_err(VectorStoreError::Tantivy)?;

        // 1. Register tokenizer.
        let code_tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .filter(AsciiFoldingFilter)
            .build();
        index
            .tokenizers()
            .register("code_tokenizer", code_tokenizer);

        // 2. Resolve fields from the new schema.
        let schema = index.schema();

        let tool_name = schema
            .get_field("tool_name")
            .map_err(|_| VectorStoreError::General("Missing tool_name field".to_string()))?;
        let description = schema
            .get_field("description")
            .map_err(|_| VectorStoreError::General("Missing description field".to_string()))?;
        let category = schema
            .get_field("category")
            .map_err(|_| VectorStoreError::General("Missing category field".to_string()))?;
        let keywords = schema
            .get_field("keywords")
            .map_err(|_| VectorStoreError::General("Missing keywords field".to_string()))?;
        let intents = schema
            .get_field("intents")
            .map_err(|_| VectorStoreError::General("Missing intents field".to_string()))?;

        // 3. Create reader.
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(VectorStoreError::Tantivy)?;

        Ok(Self {
            index,
            reader,
            writer_cache: RefCell::new(None),
            tool_name,
            description,
            category,
            keywords,
            intents,
        })
    }

    /// Create a new `KeywordIndex` or open an existing one.
    ///
    /// # Errors
    ///
    /// Returns an error when directory creation fails, index open/create fails,
    /// schema fields are missing, or reader initialization fails.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, VectorStoreError> {
        let base_path = path.as_ref();
        let index_path = base_path.join("keyword_index");
        std::fs::create_dir_all(&index_path)?;

        let meta_path = index_path.join("meta.json");

        let index = if meta_path.exists() {
            match Index::open_in_dir(&index_path) {
                Ok(idx) => idx,
                Err(_) => {
                    // Fallback: if corrupted, wipe and recreate.
                    Self::create_new_index(&index_path).map_err(VectorStoreError::Tantivy)?
                }
            }
        } else {
            Self::create_new_index(&index_path).map_err(VectorStoreError::Tantivy)?
        };

        // 1. Register tokenizer (must be done every time we open/create).
        let code_tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .filter(AsciiFoldingFilter)
            .build();
        index
            .tokenizers()
            .register("code_tokenizer", code_tokenizer);

        // 2. Resolve fields from the index schema (critical for consistency).
        let schema = index.schema();

        let tool_name = schema
            .get_field("tool_name")
            .map_err(|_| VectorStoreError::General("Missing tool_name field".to_string()))?;
        let description = schema
            .get_field("description")
            .map_err(|_| VectorStoreError::General("Missing description field".to_string()))?;
        let category = schema
            .get_field("category")
            .map_err(|_| VectorStoreError::General("Missing category field".to_string()))?;
        let keywords = schema
            .get_field("keywords")
            .map_err(|_| VectorStoreError::General("Missing keywords field".to_string()))?;
        // Check for intents field - if missing, recreate the index (schema migration).
        let Ok(intents) = schema.get_field("intents") else {
            return Self::new_with_migration(path);
        };

        // 3. Create reader with manual policy (we control reloads).
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(VectorStoreError::Tantivy)?;

        Ok(Self {
            index,
            reader,
            writer_cache: RefCell::new(None),
            tool_name,
            description,
            category,
            keywords,
            intents,
        })
    }

    /// Get the number of documents in the index.
    #[must_use]
    pub fn count_documents(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    /// Check whether index data exists.
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .join("keyword_index")
            .join("meta.json")
            .exists()
    }
}
