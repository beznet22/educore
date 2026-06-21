//! SDK error type.

use thiserror::Error;

/// Errors produced by the SDK's builder + facade methods.
#[derive(Debug, Error)]
pub enum SdkError {
    /// A required port was not provided to the builder.
    #[error("missing required port: {0}")]
    MissingPort(&'static str),

    /// A facade method delegation failed.
    #[error("facade error in {service}: {message}")]
    Facade {
        /// Which facade service produced the error.
        service: &'static str,
        /// The error message.
        message: String,
    },

    /// The underlying engine returned an error.
    #[error("engine error: {0}")]
    Engine(String),
}
