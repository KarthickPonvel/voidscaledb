// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;
use phf::phf_map;

use crate::{
    commands::exec::{
        key::{exec_del, exec_exists, exec_ttl},
        server::exec_ping,
        string::{
            exec_append, exec_get, exec_mget, exec_mset, exec_set, exec_str_decr, exec_str_decr_by,
            exec_str_incr, exec_str_incr_by,
        },
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
    Append,
    Mset,
    Mget,
    Del,
    Ttl,
    Exists,
    Incr,
    Decr,
    Incrby,
    Decrby,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExecutionPolicy {
    Local,
    SingleKey,
    MultiKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReplyKind {
    Ok,
    Integer,
    BulkString,
    Array,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReplyMergePolicy {
    None,
    IntegerSum,
    AllOk,
    OrderedArray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArgumentKind {
    None,
    SingleKey,
    Keys,
    KeyValuePairs,
    OrderedKeys,
}

#[derive(Clone, Copy)]
pub struct CommandMeta {
    pub id: CommandId,
    pub min: usize,
    pub max: Option<usize>,
    pub write: bool,

    pub execution_policy: ExecutionPolicy,
    pub argument_kind: ArgumentKind,

    pub reply_kind: ReplyKind,
    pub reply_merge_policy: ReplyMergePolicy,

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
        argument_kind: ArgumentKind,
        reply_kind: ReplyKind,
        reply_merge_policy: ReplyMergePolicy,
    ) -> Self {
        Self {
            id,
            min,
            max,
            write,
            handler,
            execution_policy: policy,
            argument_kind,
            reply_kind,
            reply_merge_policy,
        }
    }
}

pub static COMMANDS_TABLE: phf::Map<&'static [u8], CommandMeta> = phf_map! {
    b"PING" => CommandMeta::new(
        CommandId::Ping, 1, Some(2), false, exec_ping,
        ExecutionPolicy::Local, ArgumentKind::None, ReplyKind::Ok, ReplyMergePolicy::None
    ),
    b"GET"  => CommandMeta::new(
        CommandId::Get, 1, Some(1), false, exec_get,
        ExecutionPolicy::SingleKey, ArgumentKind::SingleKey, ReplyKind::BulkString, ReplyMergePolicy::None
    ),
    b"SET"  => CommandMeta::new(
        CommandId::Set, 2, Some(8), true, exec_set,
        ExecutionPolicy::SingleKey, ArgumentKind::SingleKey, ReplyKind::Ok, ReplyMergePolicy::None
    ),
    b"APPEND" => CommandMeta::new(
        CommandId::Append, 2, Some(2), true, exec_append,
        ExecutionPolicy::SingleKey, ArgumentKind::SingleKey, ReplyKind::Integer, ReplyMergePolicy::None
    ),
    b"MSET" =>CommandMeta::new(
        CommandId::Mset, 2, None, true, exec_mset,
        ExecutionPolicy::MultiKey, ArgumentKind::KeyValuePairs, ReplyKind::Ok, ReplyMergePolicy::AllOk
    ),
    b"MGET" =>CommandMeta::new(
        CommandId::Mget, 1, None, false, exec_mget,
        ExecutionPolicy::MultiKey, ArgumentKind::OrderedKeys, ReplyKind::Array, ReplyMergePolicy::OrderedArray
    ),
    b"INCR"  => CommandMeta::new(
        CommandId::Incr, 1, Some(1), true, exec_str_incr,
        ExecutionPolicy::SingleKey, ArgumentKind::SingleKey, ReplyKind::Integer, ReplyMergePolicy::None
    ),
    b"DECR"  => CommandMeta::new(
        CommandId::Decr, 1, Some(1), true, exec_str_decr,
        ExecutionPolicy::SingleKey, ArgumentKind::SingleKey, ReplyKind::Integer, ReplyMergePolicy::None
    ),
    b"INCRBY"  => CommandMeta::new(
        CommandId::Incrby, 2, Some(2), true, exec_str_incr_by,
        ExecutionPolicy::SingleKey, ArgumentKind::SingleKey, ReplyKind::Integer, ReplyMergePolicy::None
    ),
    b"DECRBY"  => CommandMeta::new(
        CommandId::Decrby, 2, Some(2), true, exec_str_decr_by,
        ExecutionPolicy::SingleKey, ArgumentKind::SingleKey, ReplyKind::Integer, ReplyMergePolicy::None
    ),
    b"DEL"  => CommandMeta::new(
        CommandId::Del, 1, None, true, exec_del,
        ExecutionPolicy::MultiKey, ArgumentKind::Keys, ReplyKind::Integer, ReplyMergePolicy::IntegerSum
    ),
    b"EXISTS"  => CommandMeta::new(
        CommandId::Exists, 1, None, false, exec_exists,
        ExecutionPolicy::MultiKey, ArgumentKind::Keys, ReplyKind::Integer, ReplyMergePolicy::IntegerSum
    ),
    b"TTL"  => CommandMeta::new(
        CommandId::Ttl, 1, Some(1), false, exec_ttl,
        ExecutionPolicy::SingleKey, ArgumentKind::SingleKey, ReplyKind::Integer, ReplyMergePolicy::None
    )
};
