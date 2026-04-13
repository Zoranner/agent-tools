//! Default [`super::WebSearchBackend`] / [`super::WebFetchBackend`] implementations.

mod direct_fetch;
mod duckduckgo;
#[cfg(feature = "tavily")]
mod tavily;

pub use direct_fetch::DirectFetchBackend;
pub use duckduckgo::DuckDuckGoSearchBackend;
#[cfg(feature = "tavily")]
pub use tavily::{TavilyFetchBackend, TavilySearchBackend};
