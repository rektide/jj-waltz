//! Upstream selection for source-only preparation.
//!
//! Selection is operation-bound. The coordinator owns working-copy snapshots,
//! filesystem proofs, and fetch/advance mutations.

pub mod source;
