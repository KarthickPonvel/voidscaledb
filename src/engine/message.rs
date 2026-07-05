// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;
use crossfire::{AsyncRx, MAsyncTx, mpsc, oneshot::TxOneshot};
use smallvec::SmallVec;

use crate::{commands::registry::CommandMeta, protocol::reply::Reply};

pub enum Message {
    Execute {
        cmd: &'static CommandMeta,
        args: SmallVec<[Bytes; 3]>,
        reply_tx: TxOneshot<Reply>,
    },
}

pub type MessageTx = MAsyncTx<mpsc::Array<Message>>;
pub type MessageRx = AsyncRx<mpsc::Array<Message>>;
