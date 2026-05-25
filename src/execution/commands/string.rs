// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::{
    engine::{
        kv::{KvStore, Record},
        value::Value,
    },
    protocol::reply::{CommandError, Reply},
};

pub fn cmd_set(kv: &mut KvStore, args: &[Bytes]) -> Reply {
    if args.len() < 2 {
        return Reply::Error(CommandError::WrongArity);
    }

    let key = args[0].clone();
    let record = Record::new(Value::String(args[1].clone()));
    kv.set(key, record);
    Reply::Ok
}

pub fn cmd_get(kv: &mut KvStore, args: &[Bytes]) -> Reply {
    if args.is_empty() {
        return Reply::Error(CommandError::WrongArity);
    }

    let key = args[0].clone();

    let record = match kv.get(&key) {
        Some(record) => record,
        None => return Reply::Null,
    };

    let val = record.value.get_bytes();

    Reply::Bulk(val.clone())
}
