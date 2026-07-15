// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::storage::StorageError;

#[derive(Debug, Clone)]
pub enum CommandError {
    UnknownCommand,
    WrongArity,
    WrongType,
    Syntax,
    OutOfRange,
    Custom(Bytes),
}

#[derive(Debug, Clone)]
pub enum Reply {
    Ok,
    Pong,
    Null,
    NullArray,
    Simple(Bytes),
    Bulk(Bytes),
    Integer(i64),
    Array(Vec<Reply>),
    Error(CommandError),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCommand => write!(f, "ERR unknown command"),
            Self::WrongArity => write!(f, "ERR wrong number of arguments"),
            Self::WrongType => write!(
                f,
                "WRONGTYPE Operation against a key holding the wrong kind of value"
            ),
            Self::Syntax => write!(f, "ERR syntax error"),
            Self::OutOfRange => write!(f, "ERR value out of range"),
            Self::Custom(msg) => write!(f, "ERR {}", String::from_utf8_lossy(msg)),
        }
    }
}

impl From<StorageError> for CommandError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::WrongType => CommandError::WrongType,
            StorageError::OutOfRange => CommandError::OutOfRange,
        }
    }
}
