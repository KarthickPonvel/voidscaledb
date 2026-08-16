// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::storage::StorageEngine;

pub trait KeyOps {
    fn del(&mut self, key: &Bytes, now: u64) -> bool;

    fn exists(&mut self, key: &Bytes, now: u64) -> bool;

    fn ttl(&mut self, key: &Bytes, now: u64) -> Option<Option<u64>>;
}

impl KeyOps for StorageEngine {
    fn del(&mut self, key: &Bytes, now: u64) -> bool {
        self.take_if_live(key, now).is_some()
    }

    fn exists(&mut self, key: &Bytes, now: u64) -> bool {
        self.peek_live(key, now).is_some()
    }

    fn ttl(&mut self, key: &Bytes, now: u64) -> Option<Option<u64>> {
        self.peek_live(key, now)
            .map(|record| record.expire_at.map(|at| at.saturating_sub(now)))
    }
}
