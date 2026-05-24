// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use crate::runtime::error::{Result, RuntimeError};

pub fn pin_thread_to_core(core_id: usize) -> Result<()> {
    match core_affinity::get_core_ids() {
        Some(ids) => match ids.into_iter().nth(core_id) {
            Some(core) => {
                core_affinity::set_for_current(core);
                Ok(())
            }
            None => Err(RuntimeError::ThreadPinFailed { id: core_id }),
        },
        None => Err(RuntimeError::ThreadPinFailed { id: core_id }),
    }
}
