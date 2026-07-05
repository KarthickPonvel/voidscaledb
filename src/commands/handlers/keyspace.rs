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
    let now = shard_engine.get_time();

    match shard_engine.get_expiry(key) {
        None => Reply::Integer(-2),
        Some(None) => Reply::Integer(-1),
        Some(Some(expiry)) => {
            if expiry <= now {
                Reply::Integer(-2)
            } else {
                let rem_ms = expiry.saturating_sub(now);
                let rem_secs = (rem_ms + 999) / 1000;
                Reply::Integer(rem_secs as i64)
            }
        }
    }
}
