// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::storage::{StorageResult, error::StorageError};

#[derive(Debug, Clone)]
pub enum Value {
    String(Bytes),
    Hash,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Hash => "hash",
        }
    }

    pub fn as_string(&self) -> StorageResult<&Bytes> {
        if let Value::String(bytes) = self {
            Ok(bytes)
        } else {
            Err(StorageError::WrongType)
        }
    }

    pub fn as_string_mut(&mut self) -> StorageResult<&mut Bytes> {
        if let Value::String(bytes) = self {
            Ok(bytes)
        } else {
            Err(StorageError::WrongType)
        }
    }
}
