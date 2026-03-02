use crate::skill_vfs::zhixing::{Error, Result};
use serde_json::json;
use std::path::Path;

use super::super::ZhixingWendaoIndexer;
use crate::{Relation, RelationType};

impl ZhixingWendaoIndexer {
    pub(in crate::skill_vfs::zhixing::indexer) fn link_task_to_agenda_document(
        &self,
        agenda_entity_name: &str,
        task_entity_name: &str,
        source_file: &Path,
        line_no: usize,
    ) -> Result<()> {
        let relation = Relation::new(
            agenda_entity_name.to_string(),
            task_entity_name.to_string(),
            RelationType::Contains,
            "Agenda document contains task entry".to_string(),
        )
        .with_source_doc(Some(source_file.display().to_string()))
        .with_metadata("source_line".to_string(), json!(line_no));
        self.graph.add_relation(&relation).map_err(|error| {
            Error::Internal(format!("Graph relation operation failed: {error}"))
        })?;
        Ok(())
    }
}
