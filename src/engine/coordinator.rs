// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::{cell::RefCell, collections::HashMap};

use bytes::Bytes;
use smallvec::SmallVec;

use crate::{
    commands::registry::{COMMANDS_TABLE, CommandMeta, ExecutionPolicy},
    engine::{
        message::{Message, MessageRx, MessageTx},
        shard::ShardEngine,
    },
    protocol::{
        command::Command,
        reply::{CommandError, Reply},
    },
};

pub struct Coordinator {
    id: usize,
    inbound: MessageRx,
    outbound: Vec<MessageTx>,
    engine: RefCell<ShardEngine>,
    shard_count: usize,
}

impl Coordinator {
    pub fn new(id: usize, inbound: MessageRx, outbound: Vec<MessageTx>) -> Self {
        let engine = RefCell::new(ShardEngine::new());
        let shard_count = outbound.len();
        Self {
            id,
            inbound,
            outbound,
            engine,
            shard_count,
        }
    }

    pub async fn run(&self) {
        while let Ok(msg) = self.inbound.recv().await {
            match msg {
                Message::Execute {
                    cmd,
                    args,
                    reply_tx,
                } => {
                    let reply = self.execute_local(cmd, args);
                    reply_tx.send(reply);
                }
            }
        }
    }

    pub async fn execute(&self, cmd: Command) -> Reply {
        let meta = match COMMANDS_TABLE.get(cmd.name()) {
            Some(meta) => meta,
            None => return Reply::Error(CommandError::UnknownCommand),
        };

        if cmd.args.len() < meta.min {
            return Reply::Error(CommandError::WrongArity);
        }
        if let Some(max) = meta.max {
            if cmd.args.len() > max {
                return Reply::Error(CommandError::WrongArity);
            }
        }

        match meta.execution_policy {
            ExecutionPolicy::Local => self.execute_local(meta, cmd.args),
            ExecutionPolicy::SingleKey => {
                let worker = self.shard_for(cmd.args[0].as_ref());
                if worker == self.id {
                    self.execute_local(meta, cmd.args)
                } else {
                    self.execute_remote(worker, meta, cmd.args).await
                }
            }
            ExecutionPolicy::MultiKey => self.execute_multi_key(meta, cmd.args).await,
        }
    }

    fn execute_local(&self, meta: &'static CommandMeta, args: SmallVec<[Bytes; 3]>) -> Reply {
        self.engine.borrow_mut().execute(meta, args)
    }

    async fn execute_remote(
        &self,
        worker: usize,
        meta: &'static CommandMeta,
        args: SmallVec<[Bytes; 3]>,
    ) -> Reply {
        let (reply_tx, reply_rx) = crossfire::oneshot::oneshot::<Reply>();

        let msg = Message::Execute {
            cmd: meta,
            args,
            reply_tx,
        };

        if self.outbound[worker].send(msg).await.is_err() {
            return Reply::Error(CommandError::Internal);
        }

        reply_rx
            .await
            .unwrap_or(Reply::Error(CommandError::Internal))
    }

    async fn execute_multi_key(
        &self,
        meta: &'static CommandMeta,
        args: SmallVec<[Bytes; 3]>,
    ) -> Reply {
        let mut groups: HashMap<usize, SmallVec<[Bytes; 8]>> = HashMap::new();

        for key in args {
            let worker = self.shard_for(key.as_ref());
            groups.entry(worker).or_default().push(key);
        }

        let mut total = 0i64;

        for (worker, keys) in groups {
            let keys: SmallVec<[Bytes; 3]> = keys.into_iter().collect();

            let reply = if worker == self.id {
                self.execute_local(meta, keys)
            } else {
                self.execute_remote(worker, meta, keys).await
            };

            if let Reply::Integer(n) = reply {
                total += n;
            }
        }

        Reply::Integer(total)
    }

    fn shard_for(&self, key: &[u8]) -> usize {
        let mut h: usize = 0xcbf29ce484222325;

        for &b in key {
            h ^= b as usize;
            h = h.wrapping_mul(0x100000001b3);
        }
        h % self.shard_count
    }
}
