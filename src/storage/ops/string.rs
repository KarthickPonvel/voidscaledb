// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::collections::hash_map::Entry;

use bytes::{Bytes, BytesMut};

use crate::{
    storage::{StorageEngine, StorageResult, error::StorageError, record::Record, value::Value},
    util::{bytes_to_i64, i64_to_bytes},
};

pub trait StringOps {
    fn get(&mut self, key: &Bytes, now: u64) -> StorageResult<Option<Bytes>>;

    fn set(
        &mut self,
        key: Bytes,
        value: Bytes,
        expire_at: Option<u64>,
        now: u64,
    ) -> StorageResult<Option<Bytes>>;

    fn append(&mut self, key: Bytes, value: Bytes, now: u64) -> StorageResult<usize>;

    fn incr_by(&mut self, key: Bytes, by: i64, now: u64) -> StorageResult<i64>;
}

impl StringOps for StorageEngine {
    fn get(&mut self, key: &Bytes, now: u64) -> StorageResult<Option<Bytes>> {
        match self.peek_live(key, now) {
            Some(record) => Ok(Some(record.value.as_string()?.clone())),
            None => Ok(None),
        }
    }

    fn set(
        &mut self,
        key: Bytes,
        value: Bytes,
        expire_at: Option<u64>,
        now: u64,
    ) -> StorageResult<Option<Bytes>> {
        match self.live_entry(key, now) {
            Entry::Vacant(v) => {
                v.insert(Record::new(Value::String(value), expire_at));
                Ok(None)
            }
            Entry::Occupied(mut o) => {
                let record = o.get_mut();
                let old = record.value.as_string()?.clone();

                record.value = Value::String(value);
                record.expire_at = expire_at;

                Ok(Some(old))
            }
        }
    }

    fn append(&mut self, key: Bytes, value: Bytes, now: u64) -> StorageResult<usize> {
        match self.live_entry(key, now) {
            Entry::Vacant(v) => {
                v.insert(Record::new(Value::String(value.clone()), None));
                Ok(value.len())
            }
            Entry::Occupied(mut o) => {
                let record = o.get_mut();
                let old_val = record.value.as_string()?;

                // TODO: Remove allocation
                let mut temp = BytesMut::from(old_val.clone());
                temp.extend_from_slice(&value);

                let len = temp.len();

                let new_val = temp.freeze();

                record.value = Value::String(new_val);
                Ok(len)
            }
        }
    }

    fn incr_by(&mut self, key: Bytes, by: i64, now: u64) -> StorageResult<i64> {
        match self.live_entry(key, now) {
            Entry::Vacant(v) => {
                v.insert(Record::new(Value::String(i64_to_bytes(by)), None));
                Ok(by)
            }
            Entry::Occupied(mut o) => {
                let record = o.get_mut();
                let old_val = record.value.as_string()?;

                let old_int = bytes_to_i64(old_val).ok_or(StorageError::OutOfRange)?;
                let new_int = old_int.checked_add(by).ok_or(StorageError::OutOfRange)?;

                record.value = Value::String(i64_to_bytes(new_int));
                Ok(new_int)
            }
        }
    }
}
