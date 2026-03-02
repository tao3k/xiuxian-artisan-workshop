mod api;
mod background_completion;
mod command_handlers;
mod command_router;
pub(crate) mod observability;
mod replies;

#[cfg(test)]
pub(in crate::channels::telegram::runtime) use api::log_preview;
pub(in crate::channels::telegram::runtime) use api::{
    handle_inbound_message_with_interrupt, push_background_completion,
};
