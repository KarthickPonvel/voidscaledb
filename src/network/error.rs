// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("connection reset by peer")]
    ConnectionReset,

    #[error("connection closed")]
    ConnectionClosed,

    #[error("connection timeout")]
    ConnectionTimeout,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("buffer overflow")]
    BufferOverflow,

    #[error("too many connections")]
    TooManyConnections,

    #[error("internal network error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, NetworkError>;
