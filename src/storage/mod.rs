// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

mod error;
mod key;
mod record;
mod result;
mod storage;
mod strings;
mod value;

pub use error::{StorageError, StorageResult};
pub use record::Record;
pub use result::WriteOutcome;
pub use storage::StorageEngine;
pub use value::Value;
