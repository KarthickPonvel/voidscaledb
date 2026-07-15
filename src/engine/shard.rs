// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::time::SystemTime;

use bytes::Bytes;
use smallvec::SmallVec;

use crate::{
    commands::{options::SetOptions, registry::CommandMeta},
    protocol::reply::Reply,
    storage::{StorageEngine, StorageResult, Value, WriteOutcome},
};

pub struct ShardEngine {
    storage: StorageEngine,
    current_time_ms: u64,
}

impl ShardEngine {
    pub fn new() -> Self {
        let storage = StorageEngine::new();
        let current_time_ms = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            storage,
            current_time_ms,
        }
    }

    #[inline(always)]
    pub fn execute(&mut self, meta: &CommandMeta, args: SmallVec<[Bytes; 3]>) -> Reply {
        self.update_time();
        (meta.handler)(self, &args)
    }

    pub fn update_time(&mut self) -> u64 {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.current_time_ms = now;
        self.current_time_ms
    }

    pub fn get_time(&self) -> u64 {
        self.current_time_ms
    }

    pub fn str_get(&mut self, key: &Bytes) -> StorageResult<Option<Bytes>> {
        let now = self.get_time();
        self.storage.str_get(key, now)
    }

    pub fn str_set(
        &mut self,
        key: Bytes,
        value: Value,
        options: SetOptions,
    ) -> StorageResult<WriteOutcome<Option<Bytes>>> {
        self.storage.str_set(key, value, options, self.get_time())
    }

    pub fn del(&mut self, key: &Bytes) -> bool {
        let now = self.get_time();
        self.storage.del(key, now)
    }

    pub fn exists(&mut self, key: &Bytes) -> bool {
        let now = self.get_time();
        self.storage.exists(key, now)
    }

    pub fn ttl(&mut self, key: &Bytes) -> Option<Option<u64>> {
        let now = self.current_time_ms;
        self.storage.ttl(key, now)
    }
}
