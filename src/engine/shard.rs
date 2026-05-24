// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;
use smallvec::SmallVec;

use crate::{
    engine::kv::KvStore,
    execution::{commands::server, registry::CommandId},
    protocol::reply::Reply,
};

pub struct ShardEngine {
    kv: KvStore,
}

impl ShardEngine {
    pub fn new() -> Self {
        let kv = KvStore::new();
        Self { kv }
    }

    #[inline(always)]
    pub fn execute(&mut self, cmd_id: CommandId, args: SmallVec<[Bytes; 3]>) -> Reply {
        match cmd_id {
            CommandId::Ping => server::cmd_ping(&mut self.kv, &args),
        }
    }
}
