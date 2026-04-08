/// Agent module for autonomous cargo-declared workflows
pub mod audit;
pub mod harness;

pub use audit::{audit_graph, AuditError, AuditReport};
