//! # educore-cli
//!
//! Sample CLI binary demonstrating consumer-side wiring of the
//! Educore engine. Provides 3 subcommands:
//!
//! - `admit` — admit a student (academic domain)
//! - `attendance` — mark a bulk attendance row (storage port)
//! - `payment` — record a payment (finance port)
//!
//! All subcommands run against an in-memory backend
//! (`educore_testkit::test_world()`). This CLI is for developer
//! ergonomics and dogfooding; it is NOT shipped to library
//! consumers.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod commands;
pub use commands::{admit, attendance, dispatch, payment};

use clap::{Parser, Subcommand};

/// The top-level CLI args.
#[derive(Debug, Parser)]
#[command(name = "educore-cli", about = "Sample Educore CLI", version)]
pub struct Cli {
    /// The subcommand.
    #[command(subcommand)]
    pub command: Command,
}

/// The 3 subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Admit a student.
    Admit {
        /// The school id (UUID).
        #[arg(long)]
        school: String,
        /// The first name.
        #[arg(long)]
        first: String,
        /// The last name.
        #[arg(long)]
        last: String,
        /// The class id (UUID).
        #[arg(long)]
        class: String,
        /// The section id (UUID).
        #[arg(long)]
        section: String,
    },
    /// Mark a bulk attendance row.
    Attendance {
        /// The school id (UUID).
        #[arg(long)]
        school: String,
        /// The student id (UUID).
        #[arg(long)]
        student: String,
        /// The date (YYYY-MM-DD).
        #[arg(long)]
        date: String,
        /// The status: "P" (present) or "A" (absent).
        #[arg(long, default_value = "P")]
        status: String,
    },
    /// Record a payment.
    Payment {
        /// The school id (UUID).
        #[arg(long)]
        school: String,
        /// The invoice id (UUID).
        #[arg(long)]
        invoice: String,
        /// The amount in minor units (e.g. cents).
        #[arg(long)]
        amount: i64,
        /// The currency code (ISO 4217, e.g. "USD").
        #[arg(long, default_value = "USD")]
        currency: String,
        /// The payment method: "cash", "card", "cheque", etc.
        #[arg(long, default_value = "cash")]
        method: String,
    },
}
