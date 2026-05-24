// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::{Bytes, BytesMut};
use smallvec::SmallVec;

use crate::protocol::{Result, command::Command, reply::Reply, resp2};

pub enum Protocol {
    RESP2,
}

pub struct Frame {
    pub expected_args: usize,
    pub current_index: usize,
    pub args: SmallVec<[Bytes; 3]>,
}

pub enum DecoderState {
    Idle,
    ReadingArrayLength,
    ReadingBulkLength,
    ReadingBulkData { remaining: usize },
}

pub struct ProtocolCodec {
    protocol: Protocol,
    state: DecoderState,
    frame: Option<Frame>,
}

impl ProtocolCodec {
    pub fn new() -> Self {
        Self {
            protocol: Protocol::RESP2,
            state: DecoderState::Idle,
            frame: None,
        }
    }

    pub fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Command>> {
        match self.protocol {
            Protocol::RESP2 => resp2::decode(buf, &mut self.state, &mut self.frame),
        }
    }

    pub fn encode(&self, buf: &mut BytesMut, reply: Reply) {
        match self.protocol {
            Protocol::RESP2 => resp2::encode(&reply, buf),
        }
    }
}
