// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::{cell::RefCell, rc::Rc};

use crate::{
    engine::shard::ShardEngine,
    network::connection::Connection,
    protocol::{
        codec::ProtocolCodec,
        reply::{CommandError, Reply},
    },
    runtime::{
        dispatch::{dispatch, dispatch_internal},
        message::Message,
        resolver::ShardResolver,
    },
};

pub async fn handle_conn(
    mut conn: Connection,
    resolver: Rc<ShardResolver>,
    engine: Rc<RefCell<ShardEngine>>,
) {
    let mut codec = ProtocolCodec::new();

    loop {
        if conn.read().await.is_err() {
            return;
        }
        loop {
            let cmd = match codec.decode(conn.rbuf_mut()) {
                Ok(Some(cmd)) => cmd,
                Ok(None) => break,
                Err(_) => {
                    let err_reply = Reply::Error(CommandError::Syntax);
                    codec.encode(conn.wbuf_mut(), err_reply);
                    return;
                }
            };
            let reply = dispatch(cmd, &resolver, &engine).await;

            codec.encode(conn.wbuf_mut(), reply);
        }
        if conn.write().await.is_err() {
            return;
        }
    }
}

pub async fn handle_inter_shard_message(msg: Message, engine: Rc<RefCell<ShardEngine>>) {
    let Message {
        cmd_id,
        args,
        tx_reply,
    } = msg;

    let reply = dispatch_internal(cmd_id, args, &engine);

    let _ = tx_reply.send(reply);
}
