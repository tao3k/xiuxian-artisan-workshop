mod identity;
mod projection;

pub(in crate::skill_vfs::zhixing::indexer) use identity::normalize_identity_token;
pub(in crate::skill_vfs::zhixing::indexer) use projection::{
    TaskLineProjection, parse_task_projection,
};
