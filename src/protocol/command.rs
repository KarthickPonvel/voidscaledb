// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;
use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: Bytes,
    pub args: SmallVec<[Bytes; 3]>,
}

impl Command {
    pub fn new(name: Bytes, args: SmallVec<[Bytes; 3]>) -> Self {
        Self { name, args }
    }

    pub fn name(&self) -> &Bytes {
        &self.name
    }

    pub fn name_str(&self) -> &str {
        std::str::from_utf8(&self.name).unwrap_or("?")
    }

    pub fn arg_len(&self) -> usize {
        self.args.len()
    }
}
