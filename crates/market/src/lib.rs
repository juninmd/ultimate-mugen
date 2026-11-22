pub mod bundle;
pub mod catalog;
pub mod client;
pub mod model;
pub mod ops;
pub mod packinfo;
pub mod persist;
pub mod portrait_cache;
pub mod refs;
pub mod repo;
pub mod safety;
mod sfx_filter;
pub mod state;
pub mod version;
pub mod zipdir;
pub mod view;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(not(target_arch = "wasm32"))]
pub mod probe;
#[cfg(target_arch = "wasm32")]
pub mod web;

pub use catalog::{Catalog, FileRef, Item, Kind, Tree, TreeEntry};
pub use client::{MarketError, RateLimit, Request, Response};
pub use persist::{MemPersist, Persist};
pub use repo::RepoRef;
pub use state::{Installed, MarketState, RepoRecord};

pub fn now_secs() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
}
