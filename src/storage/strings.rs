// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::collections::hash_map::Entry;

use bytes::Bytes;

use crate::{
    commands::options::SetOptions,
    storage::{StorageEngine, WriteOutcome, error::StorageResult, record::Record, value::Value},
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
}
