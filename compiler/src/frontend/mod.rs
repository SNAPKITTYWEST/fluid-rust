//! Frontend: Rust source → RMIR elaboration
//!
//! The frontend is responsible for:
//! - Parsing Rust syntax (using syn crate)
//! - Extracting refinement types and liquid annotations
//! - Performing ownership analysis
//! - Building the elaborated AST with ownership facts
//!
//! The elaborated AST feeds into RMIR generation.

pub mod elaboration;
pub mod ownership;

// TODO: Implement parser for refinement annotations
// TODO: Implement ownership inference engine
// TODO: Implement elaboration context and environment
// TODO: Add error reporting for ownership violations
