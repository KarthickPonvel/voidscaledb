// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use crate::runtime::error::{Result, RuntimeError};

pub fn pin_thread_to_core(core_id: usize) -> Result<()> {
    match core_affinity::get_core_ids() {
        Some(ids) if !ids.is_empty() => {
            let core = ids[core_id % ids.len()];
            core_affinity::set_for_current(core);
            Ok(())
        }
        _ => Err(RuntimeError::ThreadPinFailed { id: core_id }),
    }
}
