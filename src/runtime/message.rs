// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;
use smallvec::SmallVec;

use crate::{execution::registry::CommandId, protocol::reply::Reply};

pub struct Message {
    pub cmd_id: CommandId,
    pub args: SmallVec<[Bytes; 3]>,
    pub tx_reply: oneshot::Sender<Reply>,
}

impl Message {
    pub fn new(
        cmd_id: CommandId,
        args: SmallVec<[Bytes; 3]>,
        tx_reply: oneshot::Sender<Reply>,
    ) -> Self {
        Self {
            cmd_id,
            args,
            tx_reply,
        }
    }
}
