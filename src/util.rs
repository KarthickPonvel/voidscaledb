// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;
use std::str;

#[inline]
pub fn bytes_to_i64(b: &[u8]) -> Option<i64> {
    str::from_utf8(b).ok().and_then(|s| s.parse::<i64>().ok())
}

#[inline]
pub fn bytes_to_u64(b: &[u8]) -> Option<u64> {
    str::from_utf8(b).ok().and_then(|s| s.parse::<u64>().ok())
}

#[inline]
pub fn i64_to_bytes(val: i64) -> Bytes {
    Bytes::from(val.to_string())
}
