impl VectorStore {
    async fn open_table_or_err(&self, table_name: &str) -> Result<Dataset, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Err(VectorStoreError::TableNotFound(table_name.to_string()));
        }
        let uri = table_path.to_string_lossy();
        self.open_dataset_at_uri(uri.as_ref()).await
    }

    fn ensure_non_reserved_column(column: &str) -> Result<(), VectorStoreError> {
        if Self::is_reserved_column(column) {
            return Err(VectorStoreError::General(format!(
                "Column '{column}' is reserved and cannot be altered or dropped"
            )));
        }
        Ok(())
    }

    fn is_reserved_column(column: &str) -> bool {
        matches!(
            column,
            ID_COLUMN
                | VECTOR_COLUMN
                | CONTENT_COLUMN
                | METADATA_COLUMN
                | THREAD_ID_COLUMN
                | SKILL_NAME_COLUMN
                | CATEGORY_COLUMN
                | crate::TOOL_NAME_COLUMN
                | crate::FILE_PATH_COLUMN
                | crate::ROUTING_KEYWORDS_COLUMN
                | crate::INTENTS_COLUMN
        )
    }
}
