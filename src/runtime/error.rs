// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("worker {id} failed to start: {reason}")]
    WorkerStartFailed { id: usize, reason: String },

    #[error("worker {id} failed to pin to core {core}")]
    ThreadPinFailed { id: usize, core: usize },
}

pub type RuntimeResult<T> = std::result::Result<T, RuntimeError>;
