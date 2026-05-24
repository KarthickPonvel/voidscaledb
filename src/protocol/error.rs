// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid framing at position {position}: {reason}")]
    InvalidFrame { position: usize, reason: String },

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;
