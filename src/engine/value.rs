// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

#[derive(Debug, Clone)]
pub enum Value {
    String(Bytes),
}

impl Value {
    #[inline]
    pub fn as_string(&self) -> Option<&Bytes> {
        match self {
            Value::String(b) => Some(b),
        }
    }

    #[inline]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "string",
        }
    }
}
