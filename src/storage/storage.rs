// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::collections::hash_map::Entry;

use ahash::AHashMap;
use bytes::Bytes;

use crate::storage::record::Record;

pub struct StorageEngine {
    keyspace: AHashMap<Bytes, Record>,
}

impl StorageEngine {
    pub fn new() -> Self {
        Self {
            keyspace: AHashMap::new(),
        }
    }

    fn evict_if_expired(&mut self, key: &Bytes, now: u64) {
        match self.keyspace.get(key) {
            Some(r) if r.is_expired(now) => {
                self.keyspace.remove(key);
            }
            _ => (),
        };
    }

    pub(crate) fn peek_live(&mut self, key: &Bytes, now: u64) -> Option<&Record> {
        self.evict_if_expired(key, now);
        self.keyspace.get(key)
    }

    pub(crate) fn peek_live_mut(&mut self, key: &Bytes, now: u64) -> Option<&mut Record> {
        self.evict_if_expired(key, now);
        self.keyspace.get_mut(key)
    }

    pub(crate) fn take_if_live(&mut self, key: &Bytes, now: u64) -> Option<Record> {
        self.evict_if_expired(key, now);
        self.keyspace.remove(key)
    }

    pub(crate) fn live_entry(&mut self, key: Bytes, now: u64) -> Entry<'_, Bytes, Record> {
        self.evict_if_expired(&key, now);
        self.keyspace.entry(key)
    }
}
