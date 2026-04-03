//! Default [`super::WebSearchBackend`] / [`super::WebFetchBackend`] implementations.

mod direct_fetch;
mod duckduckgo;

pub use direct_fetch::DirectFetchBackend;
pub use duckduckgo::DuckDuckGoSearchBackend;
