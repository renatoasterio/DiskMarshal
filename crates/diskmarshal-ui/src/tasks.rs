//! Long work that runs off the interface thread.

mod feed;
mod search;
mod task;

pub(crate) use feed::Feed;
pub(crate) use search::Search;
pub(crate) use task::{Task, TaskPoll};
