//! Read-only freshening prerequisites, shared by source-only and batch workflows.
//!
//! Selection is operation-bound. The coordinator owns working-copy snapshots,
//! filesystem proofs, fetch/advance mutations, and durable recovery journals.

pub mod source;
