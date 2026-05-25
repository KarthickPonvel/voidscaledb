// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::{cell::RefCell, rc::Rc};

use bytes::Bytes;
use smallvec::SmallVec;

use crate::{
    engine::shard::ShardEngine,
    execution::registry::{CommandId, lookup},
    protocol::{
        command::Command,
        reply::{CommandError, Reply},
    },
    runtime::{
        message::Message,
        resolver::{Shard, ShardResolver},
    },
};

#[derive(Clone, Copy)]
pub enum Routing {
    Local,
    FirstKey,
    Broadcast,
}

#[inline(always)]
fn get_routing_strategy(id: CommandId) -> Routing {
    match id {
        CommandId::Ping => Routing::Local,
        CommandId::Get => Routing::FirstKey,
        CommandId::Set => Routing::FirstKey,
    }
}

pub async fn dispatch(
    cmd: Command,
    resolver: &ShardResolver,
    engine: &Rc<RefCell<ShardEngine>>,
) -> Reply {
    let cmd_meta = match lookup(&cmd.name()) {
        Some(command_meta) => command_meta,
        None => {
            return Reply::Error(CommandError::UnknownCommand);
        }
    };

    let command_id = cmd_meta.id;
    let args = cmd.args;

    let reply = match get_routing_strategy(command_id) {
        Routing::FirstKey => match resolver.resolve(&args[0]) {
            Shard::Local => engine.borrow_mut().execute(command_id, args),
            Shard::Peer(tx) => {
                let (reply_tx, reply_rx) = oneshot::channel();
                tx.send(Message::new(command_id, args, reply_tx))
                    .await
                    .unwrap();
                reply_rx.await.unwrap()
            }
        },
        Routing::Local => engine.borrow_mut().execute(command_id, args),
        _ => Reply::Ok, // TODO: Handle broadcast, multikey commands.
    };

    return reply;
}

pub fn dispatch_internal(
    command_id: CommandId,
    args: SmallVec<[Bytes; 3]>,
    engine: &Rc<RefCell<ShardEngine>>,
) -> Reply {
    engine.borrow_mut().execute(command_id, args)
}
