// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::{engine::shard::ShardEngine, protocol::reply::Reply};

#[inline]
pub fn exec_ping(_: &mut ShardEngine, args: &[Bytes]) -> Reply {
    if args.len() < 1 {
        Reply::Pong
    } else {
        Reply::Bulk(args[0].clone())
    }
}
