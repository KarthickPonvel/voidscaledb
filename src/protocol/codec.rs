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

/// Decoder state.
///
/// `Frame` lives inside the states that need it rather than as a sibling
/// `Option<Frame>` field, so "mid-command but frame == None" is not a
/// representable combination -- it cannot desync.
///
/// `ReadingRawBytes` exists for replication: a full-sync RDB payload
/// arrives inline on the same connection as the command stream, and must
/// NOT be interpreted as RESP -- it's opaque bytes. The connection layer
/// switches into this state explicitly (`ProtocolCodec::begin_raw_passthrough`)
/// once it has parsed the `$<len>` preamble itself, reads `len` bytes back
/// out via `read_raw`, and the codec returns to `Idle` on its own once
/// `remaining` reaches zero -- normal command decoding then resumes for the
/// live replication stream that follows, with no separate reset call needed.
pub enum DecoderState {
    Idle,
    ReadingArrayLength,
    ReadingBulkLength { frame: Frame },
    ReadingBulkData { frame: Frame, remaining: usize },
    ReadingRawBytes { remaining: usize },
}

pub struct ProtocolCodec {
    protocol: Protocol,
    state: DecoderState,
}

impl ProtocolCodec {
    pub fn new() -> Self {
        Self {
            protocol: Protocol::RESP2,
            state: DecoderState::Idle,
        }
    }

    /// Decodes the next complete command, if any. Returns `Ok(None)` when
    /// more bytes are needed; never consumes a partial unit from `buf`, so
    /// it's safe to call repeatedly as more bytes arrive from the socket,
    /// regardless of where the network happened to split the stream.
    pub fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Command>> {
        // If we're mid-passthrough, refuse to reinterpret raw bytes as
        // RESP -- caller must drain via read_raw() first.
        if matches!(self.state, DecoderState::ReadingRawBytes { .. }) {
            return Ok(None);
        }
        match self.protocol {
            Protocol::RESP2 => resp2::decode(buf, &mut self.state),
        }
    }

    pub fn encode(&self, buf: &mut BytesMut, reply: Reply) {
        match self.protocol {
            Protocol::RESP2 => resp2::encode(&reply, buf),
        }
    }

    pub fn begin_raw_passthrough(&mut self, len: usize) {
        debug_assert!(
            matches!(self.state, DecoderState::Idle),
            "begin_raw_passthrough called while a command was in flight"
        );
        self.state = DecoderState::ReadingRawBytes { remaining: len };
    }

    pub fn read_raw(&mut self, buf: &mut BytesMut) -> Option<Bytes> {
        let remaining = match &mut self.state {
            DecoderState::ReadingRawBytes { remaining } => remaining,
            _ => return None,
        };

        if buf.is_empty() || *remaining == 0 {
            if *remaining == 0 {
                self.state = DecoderState::Idle;
            }
            return None;
        }

        let take = buf.len().min(*remaining);
        let chunk = buf.split_to(take).freeze();
        *remaining -= take;

        if *remaining == 0 {
            self.state = DecoderState::Idle;
        }

        Some(chunk)
    }

    pub fn is_in_raw_passthrough(&self) -> bool {
        matches!(self.state, DecoderState::ReadingRawBytes { .. })
    }
}
