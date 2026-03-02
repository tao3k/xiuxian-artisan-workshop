//! Domain record models discovered by skill scanners.

mod asset;
mod data;
mod prompt;
mod reference;
mod resource;
mod template;
mod testing_record;

pub use asset::AssetRecord;
pub use data::DataRecord;
pub use prompt::PromptRecord;
pub use reference::ReferenceRecord;
pub use resource::ResourceRecord;
pub use template::TemplateRecord;
pub use testing_record::TestRecord;
