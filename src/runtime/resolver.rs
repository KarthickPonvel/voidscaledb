// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::Bytes;

use crate::runtime::message::Message;

pub enum Shard<'a> {
    Local,
    Peer(&'a kanal::AsyncSender<Message>),
}

pub struct ShardResolver {
    id: usize,
    peers: Vec<kanal::AsyncSender<Message>>,
}

impl ShardResolver {
    pub fn new(id: usize, peers: Vec<kanal::AsyncSender<Message>>) -> Self {
        Self { id, peers }
    }

    pub fn resolve(&self, key: &Bytes) -> Shard<'_> {
        let target = Self::hash(key) % self.peers.len();

        if target == self.id {
            Shard::Local
        } else {
            Shard::Peer(&self.peers[target])
        }
    }

    fn hash(key: &Bytes) -> usize {
        let mut h: usize = 0xcbf29ce484222325;

        for &b in key {
            h ^= b as usize;
            h = h.wrapping_mul(0x100000001b3);
        }

        h
    }
}
