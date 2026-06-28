// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandId {
    Ping,
    Set,
    Get,
    Del,
    Ttl,
}

pub struct CommandMeta {
    pub id: CommandId,
    pub min: usize,
    pub max: Option<usize>,
    pub write: bool,
}

#[inline(always)]
pub fn lookup(name: &[u8]) -> Option<CommandMeta> {
    match name.len() {
        4 => match name {
            b"PING" => Some(CommandMeta {
                id: CommandId::Ping,
                min: 1,
                max: Some(2),
                write: false,
            }),
            _ => None,
        },
        3 => match name {
            b"GET" => Some(CommandMeta {
                id: CommandId::Get,
                min: 1,
                max: Some(2),
                write: false,
            }),
            b"SET" => Some(CommandMeta {
                id: CommandId::Set,
                min: 2,
                max: Some(8),
                write: true,
            }),
            b"DEL" => Some(CommandMeta {
                id: CommandId::Del,
                min: 1,
                max: None,
                write: true,
            }),
            b"TTL" => Some(CommandMeta {
                id: CommandId::Ttl,
                min: 1,
                max: Some(1),
                write: false,
            }),
            _ => None,
        },
        _ => None,
    }
}
