// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;
use phf::phf_map;

use crate::{
    commands::handlers::{
        keyspace::exec_ttl,
        server::exec_ping,
        string::{exec_del, exec_get, exec_set},
    },
    engine::shard::ShardEngine,
    protocol::reply::Reply,
};

pub type Handler = fn(&mut ShardEngine, &[Bytes]) -> Reply;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandId {
    Ping,
    Set,
    Get,
    Del,
    Ttl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExecutionPolicy {
    Local,
    SingleKey,
    MultiKey,
}

#[derive(Clone, Copy)]
pub struct CommandMeta {
    pub id: CommandId,
    pub min: usize,
    pub max: Option<usize>,
    pub write: bool,
    pub execution_policy: ExecutionPolicy,
    pub handler: Handler,
}

impl CommandMeta {
    pub const fn new(
        id: CommandId,
        min: usize,
        max: Option<usize>,
        write: bool,
        handler: Handler,
        policy: ExecutionPolicy,
    ) -> Self {
        Self {
            id,
            min,
            max,
            write,
            handler,
            execution_policy: policy,
        }
    }
}

pub static COMMANDS_TABLE: phf::Map<&'static [u8], CommandMeta> = phf_map! {
    b"PING" => CommandMeta::new(CommandId::Ping, 1, Some(2), false, exec_ping, ExecutionPolicy::Local),
    b"GET"  => CommandMeta::new(CommandId::Get, 1, Some(1), false, exec_get, ExecutionPolicy::SingleKey),
    b"SET"  => CommandMeta::new(CommandId::Set, 2, Some(8), true, exec_set, ExecutionPolicy::SingleKey),
    b"DEL"  => CommandMeta::new(CommandId::Del, 1, None, true, exec_del, ExecutionPolicy::MultiKey),
    b"TTL"  => CommandMeta::new(CommandId::Ttl, 1, Some(1), false, exec_ttl, ExecutionPolicy::SingleKey),
};
