// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::{
    engine::shard::ShardEngine,
    protocol::reply::{CommandError, Reply},
};

pub fn exec_ttl(shard_engine: &mut ShardEngine, args: &[Bytes]) -> Reply {
    if args.len() != 1 {
        return Reply::Error(CommandError::WrongArity);
    }

    let key = &args[0];
    match shard_engine.ttl(key) {
        None => Reply::Integer(-2),
        Some(None) => Reply::Integer(-1),
        Some(Some(ms)) => {
            let secs = (ms + 500) / 1000;
            Reply::Integer(secs as i64)
        }
    }
}

pub fn exec_del(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    if args.is_empty() {
        return Reply::Error(CommandError::WrongArity);
    }
    let mut count = 0;
    for key in args {
        if shard.del(key) {
            count += 1;
        }
    }
    Reply::Integer(count as i64)
}
