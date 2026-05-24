// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use ahash::AHashMap;
use bytes::Bytes;

use crate::engine::value::Value;

#[derive(Debug)]
pub struct Record {
    pub value: Value,
}

impl Record {
    #[inline]
    pub fn new(value: Value) -> Self {
        Record { value }
    }
}

pub struct KvStore {
    pub map: AHashMap<Bytes, Record>,
}

impl KvStore {
    pub fn new() -> Self {
        let map = AHashMap::new();
        Self { map }
    }

    #[inline]
    pub fn get(&self, key: &Bytes) -> Option<&Record> {
        self.map.get(key)
    }

    #[inline]
    pub fn set(&mut self, key: Bytes, record: Record) -> Option<Record> {
        self.map.insert(key, record)
    }

    #[inline]
    pub fn del(&mut self, key: &Bytes) -> Option<Record> {
        self.map.remove(key)
    }

    #[inline]
    pub fn contains(&self, key: &Bytes) -> bool {
        self.map.contains_key(key)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
