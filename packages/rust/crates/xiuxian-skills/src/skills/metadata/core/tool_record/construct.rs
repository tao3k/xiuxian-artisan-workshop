use super::super::ToolAnnotations;
use super::model::{ToolEnrichment, ToolRecord};

impl ToolRecord {
    /// Creates a new `ToolRecord` with required fields.
    #[must_use]
    pub fn new(
        tool_name: String,
        description: String,
        skill_name: String,
        file_path: String,
        function_name: String,
    ) -> Self {
        Self {
            tool_name,
            description,
            skill_name,
            file_path,
            function_name,
            execution_mode: String::new(),
            keywords: Vec::new(),
            intents: Vec::new(),
            file_hash: String::new(),
            input_schema: String::new(),
            docstring: String::new(),
            category: String::new(),
            annotations: ToolAnnotations::default(),
            parameters: Vec::new(),
            skill_tools_refers: Vec::new(),
            resource_uri: String::new(),
        }
    }

    /// Creates a fully populated `ToolRecord` by applying enrichment fields.
    #[must_use]
    pub fn with_enrichment(
        tool_name: String,
        description: String,
        skill_name: String,
        file_path: String,
        function_name: String,
        enrichment: ToolEnrichment,
    ) -> Self {
        Self {
            tool_name,
            description,
            skill_name,
            file_path,
            function_name,
            execution_mode: enrichment.execution_mode,
            keywords: enrichment.keywords,
            intents: enrichment.intents,
            file_hash: enrichment.file_hash,
            input_schema: enrichment.input_schema,
            docstring: enrichment.docstring,
            category: enrichment.category,
            annotations: enrichment.annotations,
            parameters: enrichment.parameters,
            skill_tools_refers: enrichment.skill_tools_refers,
            resource_uri: enrichment.resource_uri,
        }
    }
}
