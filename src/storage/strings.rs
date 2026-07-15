// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::collections::hash_map::Entry;

use bytes::Bytes;

use crate::{
    commands::options::SetOptions,
    storage::{
        StorageEngine, StorageError, WriteOutcome, error::StorageResult, record::Record,
        value::Value,
    },
    util::{bytes_to_i64, i64_to_bytes},
};

impl StorageEngine {
    pub fn str_get(&mut self, key: &Bytes, now: u64) -> StorageResult<Option<Bytes>> {
        let record = match self.get_mut(key, now) {
            Some(record) => record,
            None => return Ok(None),
        };

        Ok(Some(record.value.as_string()?.clone()))
    }

    pub fn str_set(
        &mut self,
        key: Bytes,
        value: Value,
        options: SetOptions,
        now: u64,
    ) -> StorageResult<WriteOutcome<Option<Bytes>>> {
        match self.keyspace.entry(key) {
            Entry::Vacant(v) => {
                if options.xx {
                    return Ok(WriteOutcome::Rejected(None));
                }
                v.insert(Record::new(value, options.expires_at));
                Ok(WriteOutcome::Applied(None))
            }
            Entry::Occupied(mut o) => {
                if o.get().is_expired(now) {
                    if options.xx {
                        o.remove();
                        return Ok(WriteOutcome::Rejected(None));
                    }
                    o.insert(Record::new(value, options.expires_at));
                    return Ok(WriteOutcome::Applied(None));
                }

                let record = o.get_mut();
                let old = record.value.as_string()?.clone();

                if options.nx {
                    return Ok(WriteOutcome::Rejected(Some(old)));
                }

                let expire_at = if options.keep_ttl {
                    record.expire_at
                } else {
                    options.expires_at
                };

                record.value = value;
                record.expire_at = expire_at;

                Ok(WriteOutcome::Applied(Some(old)))
            }
        }
    }

    pub fn str_incr_decr_by(&mut self, key: Bytes, by: i64, now: u64) -> StorageResult<i64> {
        match self.keyspace.entry(key) {
            Entry::Vacant(v) => {
                v.insert(Record::new(Value::String(i64_to_bytes(by)), None));
                Ok(by)
            }
            Entry::Occupied(mut o) => {
                if o.get().is_expired(now) {
                    o.insert(Record::new(Value::String(i64_to_bytes(by)), None));
                    return Ok(by);
                }
                let record = o.get_mut();

                let old_val = record.value.as_string()?;

                let old_int = match bytes_to_i64(old_val) {
                    Some(val) => val,
                    None => return Err(StorageError::OutOfRange),
                };

                let new_int = match old_int.checked_add(by) {
                    Some(val) => val,
                    None => return Err(StorageError::OutOfRange),
                };

                record.value = Value::String(i64_to_bytes(new_int));

                return Ok(new_int);
            }
        }
    }
}
