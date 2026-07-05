// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::time::SystemTime;

use bytes::Bytes;
use smallvec::SmallVec;

use crate::{
    commands::registry::CommandMeta,
    protocol::reply::Reply,
    storage::{kv::KvStore, value::Value},
};

pub struct ShardEngine {
    kv: KvStore,
    current_time_ms: u64,
}

impl ShardEngine {
    pub fn new() -> Self {
        let kv = KvStore::new();
        let current_time_ms = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            kv,
            current_time_ms,
        }
    }

    #[inline(always)]
    pub fn execute(&mut self, meta: &CommandMeta, args: SmallVec<[Bytes; 3]>) -> Reply {
        self.update_time();
        (meta.handler)(self, &args)
    }

    #[inline]
    pub fn update_time(&mut self) -> u64 {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.current_time_ms = now;
        self.current_time_ms
    }

    #[inline]
    pub fn get_time(&self) -> u64 {
        self.current_time_ms
    }

    pub fn get_expiry(&self, key: &Bytes) -> Option<Option<u64>> {
        self.kv.get_expiry(key)
    }

    #[inline]
    pub fn kv_get(&mut self, key: &Bytes, now: u64) -> Option<Bytes> {
        self.kv.get(key, now)
    }

    pub fn kv_set(
        &mut self,
        key: Bytes,
        value: Value,
        expires_at: Option<u64>,
        keep_ttl: bool,
    ) -> Option<Bytes> {
        self.kv.update(key, value, expires_at, keep_ttl)
    }

    #[inline]
    pub fn kv_del(&mut self, key: &Bytes, now: u64) -> bool {
        self.kv.remove(key, now)
    }
}
