// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::collections::hash_map::Entry;

use ahash::AHashMap;
use bytes::Bytes;

use crate::storage::record::Record;

pub struct StorageEngine {
    pub keyspace: AHashMap<Bytes, Record>,
}

impl StorageEngine {
    pub fn new() -> Self {
        Self {
            keyspace: AHashMap::new(),
        }
    }

    pub(crate) fn get_mut(&mut self, key: &Bytes, now: u64) -> Option<&mut Record> {
        match self.keyspace.entry(key.clone()) {
            Entry::Occupied(entry) => {
                if entry.get().is_expired(now) {
                    entry.remove();
                    None
                } else {
                    Some(entry.into_mut())
                }
            }
            Entry::Vacant(_) => None,
        }
    }
}
