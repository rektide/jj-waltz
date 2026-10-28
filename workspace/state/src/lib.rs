//! Shared workspace bindings and inventory, configuration, ownership records,
//! metadata storage, snapshot/report types, and configured links.
//!
//! Shared by the lifecycle and refill crates; owns no workflow mutation of its
//! own. The metadata and ownership modules reference each other deliberately:
//! ownership validation reads managed metadata, and metadata records describe
//! owned registrations.
pub mod config;
pub mod inventory;
pub mod links;
pub mod metadata;
pub mod ownership;
pub mod snapshot;
