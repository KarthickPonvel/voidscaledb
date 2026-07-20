// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

mod error;
mod ops;
mod record;
mod result;
mod storage;
mod value;

pub use error::{StorageError, StorageResult};
pub use record::Record;
pub use result::WriteOutcome;
pub use storage::StorageEngine;
pub use value::Value;
