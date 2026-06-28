// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::{
    engine::{kv::KvStore, value::Value},
    protocol::reply::{CommandError, Reply},
};

pub fn exec_set(kv: &mut KvStore, args: &[Bytes], now: u64) -> Reply {
    if args.len() < 2 {
        return Reply::Error(CommandError::WrongArity);
    }

    let key = args[0].clone();
    let value = Value::String(args[1].clone());

    let mut nx = false;
    let mut xx = false;
    let mut get = false;
    let mut keepttl = false;
    let mut expiry_at: Option<u64> = None;

    let mut i = 2;
    while i < args.len() {
        let arg = args[i].as_ref();

        if arg.eq_ignore_ascii_case(b"NX") {
            nx = true;
        } else if arg.eq_ignore_ascii_case(b"XX") {
            xx = true;
        } else if arg.eq_ignore_ascii_case(b"GET") {
            get = true;
        } else if arg.eq_ignore_ascii_case(b"KEEPTTL") {
            keepttl = true;
        } else {
            if keepttl {
                return Reply::Error(CommandError::Syntax);
            }

            if expiry_at.is_some() {
                return Reply::Error(CommandError::Syntax);
            }
            if i + 1 >= args.len() {
                return Reply::Error(CommandError::Syntax);
            }
            let val = match parse_u64(&args[i + 1]) {
                Some(v) => v,
                None => return Reply::Error(CommandError::OutOfRange),
            };
            if arg.eq_ignore_ascii_case(b"EX") {
                expiry_at = Some(now + (val * 1000));
            } else if arg.eq_ignore_ascii_case(b"PX") {
                expiry_at = Some(now + val);
            } else if arg.eq_ignore_ascii_case(b"EXAT") {
                expiry_at = Some(val * 1000);
            } else if arg.eq_ignore_ascii_case(b"PXAT") {
                expiry_at = Some(val);
            } else {
                return Reply::Error(CommandError::Syntax);
            }
            i += 1;
        }
        i += 1;
    }

    let old_value = kv.get(&key, now);

    if nx && old_value.is_some() {
        if get {
            return Reply::Bulk(old_value.unwrap());
        }
        return Reply::Null;
    }
    if xx && !old_value.is_some() {
        return Reply::Null;
    }

    let old = kv.update(key, value, expiry_at, keepttl);

    if get {
        match old {
            Some(v) => Reply::Bulk(v),
            None => Reply::Null,
        }
    } else {
        Reply::Ok
    }
}

pub fn exec_get(kv: &mut KvStore, args: &[Bytes], now: u64) -> Reply {
    if args.is_empty() {
        return Reply::Error(CommandError::WrongArity);
    }

    let key = &args[0];

    let value = match kv.get(key, now) {
        Some(record) => record,
        None => return Reply::Null,
    };

    Reply::Bulk(value.clone())
}

pub fn exec_del(kv: &mut KvStore, args: &[Bytes], now: u64) -> Reply {
    let mut count = 0;
    for key in args {
        if kv.remove(key, now) {
            count += 1;
        }
    }
    Reply::Integer(count as i64)
}

fn parse_u64(arg: &Bytes) -> Option<u64> {
    str::from_utf8(arg).ok().and_then(|s| s.parse::<u64>().ok())
}
