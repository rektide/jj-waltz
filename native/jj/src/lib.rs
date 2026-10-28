//! Native JJ mechanisms for jj-waltz: the JJ client and command runner,
//! revision/tree queries, working-copy state access, checkout discovery, and
//! platform path helpers for external commands.
//!
//! This crate owns mechanisms only. Workspace workflow policy — creation,
//! switching, reseeding, rollback — lives in the lifecycle and refill crates.
pub mod context;
pub mod jj;
pub mod paths;
