// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Worker {id} failed to start: {reason}")]
    WorkerStartFailed { id: usize, reason: String },

    #[error("worker thread {id} failed to pin to core")]
    ThreadPinFailed { id: usize },
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
