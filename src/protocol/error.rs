// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid framing at position {position}: {reason}")]
    InvalidFrame { position: usize, reason: String },

    #[error("Unsupported Protocol")]
    UnsupportedProtocol,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;
