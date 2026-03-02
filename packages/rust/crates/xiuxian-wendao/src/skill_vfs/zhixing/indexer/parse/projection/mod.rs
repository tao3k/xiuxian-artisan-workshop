mod metadata_parser;
mod task_line_parser;

pub(in crate::skill_vfs::zhixing::indexer) use task_line_parser::{
    TaskLineProjection, parse_task_projection,
};
