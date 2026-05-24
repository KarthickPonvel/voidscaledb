// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::{engine::kv::KvStore, protocol::reply::Reply};

#[inline]
pub fn cmd_ping(_: &mut KvStore, args: &[Bytes]) -> Reply {
    if args.len() < 1 {
        Reply::Pong
    } else {
        Reply::Bulk(args[0].clone())
    }
}
