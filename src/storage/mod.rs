// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

mod error;
mod ops;
mod record;
mod storage;
mod value;

pub use error::{StorageError, StorageResult};
pub use ops::key::KeyOps;
pub use ops::string::StringOps;
pub use record::Record;
pub use storage::StorageEngine;
pub use value::Value;
