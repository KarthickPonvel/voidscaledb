// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::time::SystemTime;

use bytes::Bytes;
use smallvec::SmallVec;

use crate::{commands::registry::CommandMeta, protocol::reply::Reply, storage::StorageEngine};

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

    pub fn storage(&mut self) -> &mut StorageEngine {
        &mut self.storage
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
}
