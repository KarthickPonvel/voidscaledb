// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::{cell::RefCell, collections::HashMap};

use bytes::Bytes;
use smallvec::SmallVec;

use crate::{
    commands::registry::{
        ArgumentKind, COMMANDS_TABLE, CommandMeta, ExecutionPolicy, ReplyMergePolicy,
    },
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
        match meta.argument_kind {
            ArgumentKind::Keys => self.execute_multi_key_keys(meta, args).await,

            ArgumentKind::KeyValuePairs => self.execute_multi_key_pairs(meta, args).await,

            ArgumentKind::OrderedKeys => self.execute_multi_key_ordered(meta, args).await,

            _ => unreachable!("invalid argument kind for MultiKey"),
        }
    }

    async fn execute_multi_key_pairs(
        &self,
        meta: &'static CommandMeta,
        args: SmallVec<[Bytes; 3]>,
    ) -> Reply {
        if args.len() % 2 != 0 {
            return Reply::Error(CommandError::WrongArity);
        }

        let mut groups: HashMap<usize, SmallVec<[Bytes; 8]>> = HashMap::new();

        let mut iter = args.into_iter();

        while let Some(key) = iter.next() {
            let value = iter.next().unwrap();

            let worker = self.shard_for(key.as_ref());

            let group = groups.entry(worker).or_default();

            group.push(key);
            group.push(value);
        }

        let mut replies = Vec::with_capacity(groups.len());

        for (worker, args) in groups {
            let args: SmallVec<[Bytes; 3]> = args.into_iter().collect();

            let reply = if worker == self.id {
                self.execute_local(meta, args)
            } else {
                self.execute_remote(worker, meta, args).await
            };

            replies.push(reply);
        }

        self.merge_multi_key_reply(meta, replies)
    }

    async fn execute_multi_key_keys(
        &self,
        meta: &'static CommandMeta,
        args: SmallVec<[Bytes; 3]>,
    ) -> Reply {
        let mut groups: HashMap<usize, SmallVec<[Bytes; 8]>> = HashMap::new();

        for key in args {
            let worker = self.shard_for(key.as_ref());

            groups.entry(worker).or_default().push(key);
        }

        let mut replies = Vec::with_capacity(groups.len());

        for (worker, keys) in groups {
            let keys: SmallVec<[Bytes; 3]> = keys.into_iter().collect();

            let reply = if worker == self.id {
                self.execute_local(meta, keys)
            } else {
                self.execute_remote(worker, meta, keys).await
            };

            replies.push(reply);
        }

        self.merge_multi_key_reply(meta, replies)
    }

    async fn execute_multi_key_ordered(
        &self,
        meta: &'static CommandMeta,
        args: SmallVec<[Bytes; 3]>,
    ) -> Reply {
        let mut groups: HashMap<usize, Vec<(usize, Bytes)>> = HashMap::new();

        for (index, key) in args.into_iter().enumerate() {
            let worker = self.shard_for(key.as_ref());

            groups.entry(worker).or_default().push((index, key));
        }

        let mut results = Vec::new();

        for (worker, keys) in groups {
            let reply = if worker == self.id {
                self.execute_local_ordered(meta, keys)
            } else {
                self.execute_remote_ordered(worker, meta, keys).await
            };

            results.extend(reply);
        }

        self.merge_ordered_array(results)
    }

    fn execute_local_ordered(
        &self,
        meta: &'static CommandMeta,
        keys: Vec<(usize, Bytes)>,
    ) -> Vec<(usize, Reply)> {
        let args: SmallVec<[Bytes; 3]> = keys.iter().map(|(_, key)| key.clone()).collect();

        let reply = self.execute_local(meta, args);

        match reply {
            Reply::Array(values) => keys
                .into_iter()
                .zip(values)
                .map(|((index, _), value)| (index, value))
                .collect(),

            Reply::Error(err) => keys
                .into_iter()
                .map(|(index, _)| (index, Reply::Error(err.clone())))
                .collect(),

            _ => Vec::new(),
        }
    }
    async fn execute_remote_ordered(
        &self,
        worker: usize,
        meta: &'static CommandMeta,
        keys: Vec<(usize, Bytes)>,
    ) -> Vec<(usize, Reply)> {
        let args: SmallVec<[Bytes; 3]> = keys.iter().map(|(_, key)| key.clone()).collect();

        let reply = self.execute_remote(worker, meta, args).await;

        match reply {
            Reply::Array(values) => keys
                .into_iter()
                .zip(values)
                .map(|((index, _), value)| (index, value))
                .collect(),

            Reply::Error(err) => keys
                .into_iter()
                .map(|(index, _)| (index, Reply::Error(err.clone())))
                .collect(),

            _ => Vec::new(),
        }
    }

    fn shard_for(&self, key: &[u8]) -> usize {
        let mut h: usize = 0xcbf29ce484222325;

        for &b in key {
            h ^= b as usize;
            h = h.wrapping_mul(0x100000001b3);
        }
        h % self.shard_count
    }

    fn merge_multi_key_reply(&self, meta: &'static CommandMeta, replies: Vec<Reply>) -> Reply {
        match meta.reply_merge_policy {
            ReplyMergePolicy::IntegerSum => {
                let mut total = 0i64;

                for reply in replies {
                    match reply {
                        Reply::Integer(n) => {
                            total += n;
                        }
                        Reply::Error(err) => {
                            return Reply::Error(err);
                        }
                        _ => {
                            return Reply::Error(CommandError::Internal);
                        }
                    }
                }
                Reply::Integer(total)
            }

            ReplyMergePolicy::AllOk => {
                for reply in replies {
                    match reply {
                        Reply::Ok => {}
                        Reply::Error(err) => {
                            return Reply::Error(err);
                        }
                        _ => {
                            return Reply::Error(CommandError::Internal);
                        }
                    }
                }
                Reply::Ok
            }

            ReplyMergePolicy::OrderedArray => {
                unreachable!("ArrayOrdered must use execute_multi_key_ordered")
            }

            ReplyMergePolicy::None => {
                unreachable!("Multi-key command requires a reply merge policy")
            }
        }
    }

    fn merge_ordered_array(&self, results: Vec<(usize, Reply)>) -> Reply {
        let len = results.len();

        let mut ordered: Vec<Option<Reply>> = (0..len).map(|_| None).collect();

        for (index, reply) in results {
            ordered[index] = Some(reply);
        }

        Reply::Array(
            ordered
                .into_iter()
                .map(|reply| reply.expect("missing ordered reply"))
                .collect(),
        )
    }
}
