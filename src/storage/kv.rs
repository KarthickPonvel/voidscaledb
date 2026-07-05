// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::collections::hash_map::Entry;

use ahash::AHashMap;
use bytes::Bytes;

use crate::storage::value::Value;

#[derive(Debug)]
struct Record {
    pub value: Value,
    pub expires_at: Option<u64>,
}

impl Record {
    #[inline]
    pub fn is_expired(&self, now: u64) -> bool {
        match self.expires_at {
            Some(expiry) => expiry <= now,
            None => false,
        }
    }
}

pub struct KvStore {
    map: AHashMap<Bytes, Record>,
}

impl KvStore {
    pub fn new() -> Self {
        let map = AHashMap::new();
        Self { map }
    }

    fn resolve(&mut self, key: &Bytes, now: u64) -> Option<&Record> {
        let expired = match self.map.get(key) {
            Some(record) => record.is_expired(now),
            None => return None,
        };

        if expired {
            self.map.remove(key);
            return None;
        }

        self.map.get(key)
    }

    #[inline]
    pub fn get(&mut self, key: &Bytes, now: u64) -> Option<Bytes> {
        match self.resolve(key, now) {
            Some(record) => Some(record.value.get_bytes().clone()),
            None => None,
        }
    }

    #[inline]
    pub fn update(
        &mut self,
        key: Bytes,
        value: Value,
        expires_at: Option<u64>,
        keep_ttl: bool,
    ) -> Option<Bytes> {
        let existing_expiry = self.map.get(&key).and_then(|r| r.expires_at);

        let new_expiry = if keep_ttl {
            existing_expiry
        } else {
            expires_at
        };
        match self.map.entry(key) {
            Entry::Occupied(mut entry) => {
                let old_record = entry.get();
                let old_val = old_record.value.get_bytes().clone();
                entry.insert(Record {
                    value,
                    expires_at: new_expiry,
                });
                Some(old_val)
            }
            Entry::Vacant(entry) => {
                entry.insert(Record {
                    value,
                    expires_at: new_expiry,
                });
                None
            }
        }
    }

    #[inline]
    pub fn remove(&mut self, key: &Bytes, now: u64) -> bool {
        if let Some(record) = self.map.remove(key) {
            if record.is_expired(now) {
                self.map.remove(key);
                return false;
            }

            self.map.remove(key);
            return true;
        }
        false
    }

    pub fn get_expiry(&self, key: &Bytes) -> Option<Option<u64>> {
        self.map.get(key).map(|r| r.expires_at)
    }
}
