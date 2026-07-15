// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::storage::StorageEngine;

impl StorageEngine {
    pub fn ttl(&mut self, key: &Bytes, now: u64) -> Option<Option<u64>> {
        match self.keyspace.get(key) {
            Some(r) if r.is_expired(now) => {
                self.keyspace.remove(key);
                None
            }
            Some(r) => Some(r.expire_at.map(|at| at.saturating_sub(now))),
            None => None,
        }
    }

    pub fn del(&mut self, key: &Bytes, now: u64) -> bool {
        match self.keyspace.remove(key) {
            Some(r) => !r.is_expired(now),
            None => false,
        }
    }
}
