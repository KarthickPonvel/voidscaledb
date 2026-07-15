// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

#[derive(Debug, Default, Clone)]
pub struct SetOptions {
    pub nx: bool,
    pub xx: bool,
    pub get: bool,
    pub keep_ttl: bool,
    pub expires_at: Option<u64>,
}
