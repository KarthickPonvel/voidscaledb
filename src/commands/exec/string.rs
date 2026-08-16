// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::{
    commands::options::SetOptions,
    engine::shard::ShardEngine,
    protocol::reply::{CommandError, Reply},
    storage::{KeyOps, StringOps},
    util::bytes_to_i64,
};

pub fn exec_get(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    if args.len() != 1 {
        return Reply::Error(CommandError::WrongArity);
    }
    let now = shard.get_time();
    match shard.storage().get(&args[0], now) {
        Ok(Some(v)) => Reply::Bulk(v),
        Ok(None) => Reply::Null,
        Err(e) => Reply::Error(e.into()),
    }
}

pub fn exec_set(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    let now = shard.get_time();

    let (key, value, options) = match parse_set_args(args, now) {
        Ok(parsed) => parsed,
        Err(e) => return Reply::Error(e),
    };

    if options.nx && options.xx {
        return Reply::Error(CommandError::Syntax);
    }
    if options.keep_ttl && options.expires_at.is_some() {
        return Reply::Error(CommandError::Syntax);
    }

    if options.nx && shard.storage().exists(&key, now) {
        return no_write_reply(shard, &key, now, options.get);
    }
    if options.xx && !shard.storage().exists(&key, now) {
        return no_write_reply(shard, &key, now, options.get);
    }

    let expire_at = if options.keep_ttl {
        match shard.storage().ttl(&key, now) {
            Some(Some(remaining)) => Some(now + remaining),
            _ => None,
        }
    } else {
        options.expires_at
    };

    let old = match shard.storage().set(key, value, expire_at, now) {
        Ok(old) => old,
        Err(e) => return Reply::Error(e.into()),
    };

    if options.get {
        old.map(Reply::Bulk).unwrap_or(Reply::Null)
    } else {
        Reply::Ok
    }
}

fn no_write_reply(shard: &mut ShardEngine, key: &Bytes, now: u64, get: bool) -> Reply {
    if !get {
        return Reply::Null;
    }
    match shard.storage().get(key, now) {
        Ok(v) => v.map(Reply::Bulk).unwrap_or(Reply::Null),
        Err(e) => Reply::Error(e.into()),
    }
}

fn parse_set_args(args: &[Bytes], now: u64) -> Result<(Bytes, Bytes, SetOptions), CommandError> {
    if args.len() < 2 {
        return Err(CommandError::WrongArity);
    }

    let key = args[0].clone();
    let value = args[1].clone();
    let mut options = SetOptions::default();

    let mut i = 2;
    while i < args.len() {
        let token = &args[i];
        if token.eq_ignore_ascii_case(b"NX") {
            options.nx = true;
            i += 1;
        } else if token.eq_ignore_ascii_case(b"XX") {
            options.xx = true;
            i += 1;
        } else if token.eq_ignore_ascii_case(b"GET") {
            options.get = true;
            i += 1;
        } else if token.eq_ignore_ascii_case(b"KEEPTTL") {
            options.keep_ttl = true;
            i += 1;
        } else if token.eq_ignore_ascii_case(b"EX") {
            let seconds = next_i64(args, i)?.max(0) as u64;
            options.expires_at = Some(now.saturating_add(seconds.saturating_mul(1000)));
            i += 2;
        } else if token.eq_ignore_ascii_case(b"PX") {
            let millis = next_i64(args, i)?.max(0) as u64;
            options.expires_at = Some(now.saturating_add(millis));
            i += 2;
        } else if token.eq_ignore_ascii_case(b"EXAT") {
            let seconds = next_i64(args, i)?.max(0) as u64;
            options.expires_at = Some(seconds.saturating_mul(1000));
            i += 2;
        } else if token.eq_ignore_ascii_case(b"PXAT") {
            let millis = next_i64(args, i)?.max(0) as u64;
            options.expires_at = Some(millis);
            i += 2;
        } else {
            return Err(CommandError::Syntax);
        }
    }

    Ok((key, value, options))
}

fn next_i64(args: &[Bytes], flag_index: usize) -> Result<i64, CommandError> {
    let raw = args.get(flag_index + 1).ok_or(CommandError::Syntax)?;
    bytes_to_i64(raw).ok_or(CommandError::OutOfRange)
}

pub fn exec_mset(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    if args.is_empty() || args.len() % 2 != 0 {
        return Reply::Error(CommandError::WrongArity);
    }
    let now = shard.get_time();
    let mut pairs = args.chunks_exact(2);
    for pair in &mut pairs {
        if let Err(e) = shard
            .storage()
            .set(pair[0].clone(), pair[1].clone(), None, now)
        {
            return Reply::Error(e.into());
        }
    }
    Reply::Ok
}

pub fn exec_mget(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    let now = shard.get_time();
    let mut out = Vec::with_capacity(args.len());
    for key in args {
        match shard.storage().get(key, now) {
            Ok(Some(v)) => out.push(Reply::Bulk(v)),
            Ok(None) => out.push(Reply::Null),
            Err(_) => out.push(Reply::Null),
        }
    }
    Reply::Array(out)
}

pub fn exec_str_incr(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    incr_reply(shard, args, 1)
}

pub fn exec_str_decr(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    incr_reply(shard, args, -1)
}

pub fn exec_str_incr_by(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    incr_by_reply(shard, args, 1)
}

pub fn exec_str_decr_by(shard: &mut ShardEngine, args: &[Bytes]) -> Reply {
    incr_by_reply(shard, args, -1)
}

fn incr_reply(shard: &mut ShardEngine, args: &[Bytes], by: i64) -> Reply {
    if args.len() != 1 {
        return Reply::Error(CommandError::WrongArity);
    }
    let now = shard.get_time();
    match shard.storage().incr_by(args[0].clone(), by, now) {
        Ok(n) => Reply::Integer(n),
        Err(e) => Reply::Error(e.into()),
    }
}

fn incr_by_reply(shard: &mut ShardEngine, args: &[Bytes], sign: i64) -> Reply {
    if args.len() != 2 {
        return Reply::Error(CommandError::WrongArity);
    }
    let amount = match bytes_to_i64(&args[1]) {
        Some(v) => v,
        None => return Reply::Error(CommandError::OutOfRange),
    };
    let by = match amount.checked_mul(sign) {
        Some(v) => v,
        None => return Reply::Error(CommandError::OutOfRange),
    };
    let now = shard.get_time();
    match shard.storage().incr_by(args[0].clone(), by, now) {
        Ok(n) => Reply::Integer(n),
        Err(e) => Reply::Error(e.into()),
    }
}
