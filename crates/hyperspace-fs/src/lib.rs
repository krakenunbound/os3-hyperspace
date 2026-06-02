//! Hyperspace FS — objects and dimensions as addressable space.
//!
//! The in-memory store is a development stand-in until Redox-side persistence
//! is implemented.

mod file;
mod memory;
mod path;
mod store;

pub use file::JsonWorkspaceStore;
pub use memory::InMemoryObjectStore;
pub use path::HyperspacePath;
pub use store::ObjectStore;
