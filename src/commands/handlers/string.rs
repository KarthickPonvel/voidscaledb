// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::{
    commands::options::SetOptions,
    engine::shard::ShardEngine,
    protocol::reply::{CommandError, Reply},
    storage::{Value, WriteOutcome},
};

pub fn exec_set(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    if args.len() < 2 {
        return Reply::Error(CommandError::WrongArity);
    }

    let key = args[0].clone();
    let value = Value::String(args[1].clone());
    let now = shard.get_time();

    let mut options = SetOptions::default();
    let mut get = false;

    let mut i = 2;
    while i < args.len() {
        let arg = args[i].as_ref();

        if arg.eq_ignore_ascii_case(b"NX") {
            options.nx = true;
        } else if arg.eq_ignore_ascii_case(b"XX") {
            options.xx = true;
        } else if arg.eq_ignore_ascii_case(b"GET") {
            get = true;
        } else if arg.eq_ignore_ascii_case(b"KEEPTTL") {
            if options.expires_at.is_some() {
                return Reply::Error(CommandError::Syntax);
            }
            options.keep_ttl = true;
        } else {
            if options.keep_ttl || options.expires_at.is_some() {
                return Reply::Error(CommandError::Syntax);
            }
            if i + 1 >= args.len() {
                return Reply::Error(CommandError::Syntax);
            }
            let val = match parse_u64(&args[i + 1]) {
                Some(v) => v,
                None => return Reply::Error(CommandError::OutOfRange),
            };
            options.expires_at = Some(if arg.eq_ignore_ascii_case(b"EX") {
                now + val * 1000
            } else if arg.eq_ignore_ascii_case(b"PX") {
                now + val
            } else if arg.eq_ignore_ascii_case(b"EXAT") {
                val * 1000
            } else if arg.eq_ignore_ascii_case(b"PXAT") {
                val
            } else {
                return Reply::Error(CommandError::Syntax);
            });
            i += 1;
        }
        i += 1;
    }

    if options.nx && options.xx {
        return Reply::Error(CommandError::Syntax);
    }

    match shard.str_set(key, value, options) {
        Ok(WriteOutcome::Applied(old)) => {
            if get {
                old.map_or(Reply::Null, Reply::Bulk)
            } else {
                Reply::Ok
            }
        }
        Ok(WriteOutcome::Rejected(old)) => {
            if get {
                old.map_or(Reply::Null, Reply::Bulk)
            } else {
                Reply::Null
            }
        }
        Err(e) => Reply::Error(e.into()),
    }
}

pub fn exec_get(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    if args.is_empty() {
        return Reply::Error(CommandError::WrongArity);
    }
    let key = &args[0];
    match shard.str_get(key) {
        Ok(Some(v)) => Reply::Bulk(v),
        Ok(None) => Reply::Null,
        Err(e) => Reply::Error(e.into()),
    }
}

fn parse_u64(arg: &Bytes) -> Option<u64> {
    str::from_utf8(arg).ok().and_then(|s| s.parse::<u64>().ok())
}
