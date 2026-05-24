// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandId {
    Ping,
}

pub struct CommandMeta {
    pub id: CommandId,
    pub min: usize,
    pub max: usize,
    pub write: bool,
}

#[inline(always)]
pub fn lookup(name: &[u8]) -> Option<CommandMeta> {
    match name.len() {
        4 => match name {
            b"PING" => Some(CommandMeta {
                id: CommandId::Ping,
                min: 1,
                max: 2,
                write: false,
            }),
            _ => None,
        },
        _ => None,
    }
}
