//! # Finance domain errors
//!
//! Phase 7 aliases the engine's [`DomainError`] as `FinanceError`.
//! The finance services return `Result<T, DomainError>`; the alias
//! is for consumer code that prefers the domain-scoped name.

#![allow(missing_docs)]

pub use educore_core::error::DomainError as FinanceError;
pub use educore_core::error::Result;
