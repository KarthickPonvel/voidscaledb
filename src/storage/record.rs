// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use crate::storage::value::Value;

#[derive(Debug)]
pub struct Record {
    pub value: Value,
    pub expire_at: Option<u64>,
}

impl Record {
    pub fn new(value: Value, expire_at: Option<u64>) -> Self {
        Self { value, expire_at }
    }

    pub fn is_expired(&self, now: u64) -> bool {
        match self.expire_at {
            Some(ex) => ex <= now,
            None => false,
        }
    }
}
