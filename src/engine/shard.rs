// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::time::SystemTime;

use bytes::Bytes;
use smallvec::SmallVec;

use crate::{
    engine::kv::KvStore,
    execution::{
        commands::{keyspace, server, string},
        registry::CommandId,
    },
    protocol::reply::Reply,
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
    pub fn execute(&mut self, cmd_id: CommandId, args: SmallVec<[Bytes; 3]>) -> Reply {
        let now = self.update_time();
        match cmd_id {
            CommandId::Ping => server::exec_ping(&mut self.kv, &args),
            CommandId::Get => string::exec_get(&mut self.kv, &args, now),
            CommandId::Set => string::exec_set(&mut self.kv, &args, now),
            CommandId::Del => string::exec_del(&mut self.kv, &args, now),
            CommandId::Ttl => keyspace::exec_ttl(&mut self.kv, &args, now),
        }
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
}
