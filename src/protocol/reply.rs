// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

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

impl Reply {
    #[inline]
    pub fn arity() -> Self {
        Reply::Error(CommandError::WrongArity)
    }

    #[inline]
    pub fn unknown() -> Self {
        Reply::Error(CommandError::UnknownCommand)
    }

    #[inline]
    pub fn err(msg: impl Into<Bytes>) -> Self {
        Reply::Error(CommandError::Custom(msg.into()))
    }

    #[inline]
    pub fn bulk(data: impl Into<Bytes>) -> Self {
        Reply::Bulk(data.into())
    }

    #[inline]
    pub fn wrong_type() -> Self {
        Reply::Error(CommandError::WrongType)
    }

    #[inline]
    pub fn syntax() -> Self {
        Reply::Error(CommandError::Syntax)
    }

    #[inline]
    pub fn simple(msg: impl Into<Bytes>) -> Self {
        Reply::Simple(msg.into())
    }
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
