// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

pub enum WriteOutcome<T> {
    Applied(T),
    Rejected(T),
}
