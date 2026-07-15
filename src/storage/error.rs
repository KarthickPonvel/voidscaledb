// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("value is not the expected type")]
    WrongType,

    #[error("value is out of the allowed range")]
    OutOfRange,
}

pub type StorageResult<T> = std::result::Result<T, StorageError>;
