// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use bytes::{Buf, Bytes, BytesMut};
use smallvec::SmallVec;

use crate::protocol::{
    Result,
    codec::{DecoderState, Frame},
    command::Command,
    error::ProtocolError,
};

const MAX_ARRAY_LEN: usize = 1024 * 1024; // max number of array arguments per command
const MAX_BULK_LEN: usize = 512 * 1024 * 1024; // max size of a single bulk string

pub fn decode(buf: &mut BytesMut, state: &mut DecoderState) -> Result<Option<Command>> {
    loop {
        match state {
            DecoderState::Idle => {
                if buf.is_empty() {
                    return Ok(None);
                }

                if buf[0] == b'*' {
                    *state = DecoderState::ReadingArrayLength;
                } else {
                    return decode_inline(buf);
                }
            }

            DecoderState::ReadingArrayLength => match decode_len(buf, b'*', MAX_ARRAY_LEN)? {
                Some((count, consumed)) => {
                    buf.advance(consumed);

                    if count == 0 {
                        *state = DecoderState::Idle;
                        return Err(ProtocolError::InvalidFrame {
                            position: 0,
                            reason: "Empty command (zero-length array)".into(),
                        });
                    }

                    *state = DecoderState::ReadingBulkLength {
                        frame: Frame {
                            expected_args: count,
                            current_index: 0,
                            args: SmallVec::with_capacity(count.min(16)),
                        },
                    };
                }
                None => return Ok(None),
            },

            DecoderState::ReadingBulkLength { frame } => {
                match decode_len(buf, b'$', MAX_BULK_LEN)? {
                    Some((len, consumed)) => {
                        buf.advance(consumed);
                        let frame = take_frame(frame);
                        *state = DecoderState::ReadingBulkData {
                            frame,
                            remaining: len,
                        };
                    }
                    None => return Ok(None),
                }
            }

            DecoderState::ReadingBulkData { frame, remaining } => {
                let total = *remaining + 2;
                if buf.len() < total {
                    return Ok(None);
                }

                if &buf[*remaining..*remaining + 2] != b"\r\n" {
                    return Err(ProtocolError::InvalidFrame {
                        position: *remaining,
                        reason: "Missing CRLF after bulk data".into(),
                    });
                }

                let data = buf.split_to(*remaining).freeze();
                buf.advance(2);

                frame.args.push(data);
                frame.current_index += 1;

                if frame.current_index == frame.expected_args {
                    let frame = match std::mem::replace(state, DecoderState::Idle) {
                        DecoderState::ReadingBulkData { frame, .. } => frame,
                        _ => unreachable!("state was just matched as ReadingBulkData"),
                    };

                    let mut args = frame.args;
                    if args.is_empty() {
                        return Err(ProtocolError::InvalidFrame {
                            position: 0,
                            reason: "Empty command (zero-length array)".into(),
                        });
                    }
                    let mut name = args.remove(0);

                    // COPY: Allocates a Vec<u8> of size of 'name'.
                    let mut name_vec = name.to_vec();
                    name_vec.make_ascii_uppercase();
                    name = Bytes::from(name_vec);

                    return Ok(Some(Command::new(name, args)));
                }

                let frame = take_frame(frame);
                *state = DecoderState::ReadingBulkLength { frame };
            }

            DecoderState::ReadingRawBytes { .. } => {
                return Ok(None);
            }
        }
    }
}

fn take_frame(frame: &mut Frame) -> Frame {
    std::mem::replace(
        frame,
        Frame {
            expected_args: 0,
            current_index: 0,
            args: SmallVec::new(),
        },
    )
}

fn decode_len(buf: &[u8], expect_prefix: u8, max_value: usize) -> Result<Option<(usize, usize)>> {
    if buf.is_empty() {
        return Ok(None);
    }

    if buf[0] != expect_prefix {
        return Err(ProtocolError::InvalidFrame {
            position: 0,
            reason: format!("Expected '{}'", expect_prefix as char),
        });
    }

    let nl = match buf.iter().position(|&b| b == b'\n') {
        Some(p) => p,
        None => return Ok(None),
    };

    if nl < 2 || buf[nl - 1] != b'\r' {
        return Err(ProtocolError::InvalidFrame {
            position: nl,
            reason: "Missing CR before LF".into(),
        });
    }

    let digits = &buf[1..nl - 1];
    if digits.is_empty() {
        return Err(ProtocolError::InvalidFrame {
            position: 1,
            reason: "Empty length field".into(),
        });
    }

    let val = std::str::from_utf8(digits)
        .map_err(|_| ProtocolError::InvalidFrame {
            position: 1,
            reason: "Invalid UTF8 in length field".into(),
        })?
        .parse::<usize>()
        .map_err(|_| ProtocolError::InvalidFrame {
            position: 1,
            reason: "Length is not a non-negative integer".into(),
        })?;

    if val > max_value {
        return Err(ProtocolError::ConstraintViolation(format!(
            "Value {} exceeds limit {}",
            val, max_value
        )));
    }

    Ok(Some((val, nl + 1)))
}

fn decode_inline(buf: &mut BytesMut) -> Result<Option<Command>> {
    let end = match buf.iter().position(|&b| b == b'\n') {
        Some(p) => p,
        None => return Ok(None),
    };

    if end == 0 {
        buf.advance(1);
        return Err(ProtocolError::InvalidFrame {
            position: 0,
            reason: "Empty line".into(),
        });
    }

    let mut line = buf.split_to(end + 1);
    line.truncate(line.len() - 1);
    if line.last() == Some(&b'\r') {
        line.truncate(line.len() - 1);
    }

    let name_end = line.iter().position(|&b| b == b' ').unwrap_or(line.len());
    if name_end == 0 {
        return Err(ProtocolError::InvalidFrame {
            position: end,
            reason: "Empty command".into(),
        });
    }

    line[..name_end].make_ascii_uppercase();
    let name = line.split_to(name_end).freeze();

    let mut tokens: SmallVec<[Bytes; 3]> = SmallVec::new();
    while !line.is_empty() {
        let start = line.iter().position(|&b| b != b' ').unwrap_or(line.len());
        line.advance(start);

        if line.is_empty() {
            break;
        }

        let end = line.iter().position(|&b| b == b' ').unwrap_or(line.len());
        tokens.push(line.split_to(end).freeze());
    }

    Ok(Some(Command::new(name, tokens)))
}

#[cfg(test)]
mod tests {
    use crate::protocol::codec::ProtocolCodec;
    use bytes::BytesMut;

    fn buf(s: &str) -> BytesMut {
        BytesMut::from(s.as_bytes())
    }

    fn once(input: &str) -> crate::protocol::Result<Option<crate::protocol::command::Command>> {
        let mut codec = ProtocolCodec::new();
        let mut b = buf(input);
        codec.decode(&mut b)
    }

    #[test]
    fn resume_split_right_after_array_prefix() {
        let mut codec = ProtocolCodec::new();
        let mut b = buf("*");
        assert!(codec.decode(&mut b).unwrap().is_none());
        assert_eq!(&b[..], b"*");

        b.extend_from_slice(b"1\r\n$4\r\nPING\r\n");
        let c = codec.decode(&mut b).unwrap().unwrap();
        assert_eq!(c.name_str(), "PING");
    }

    #[test]
    fn resume_split_right_after_bulk_prefix() {
        let mut codec = ProtocolCodec::new();
        let mut b = buf("*1\r\n$");
        assert!(codec.decode(&mut b).unwrap().is_none());

        b.extend_from_slice(b"4\r\nPING\r\n");
        let c = codec.decode(&mut b).unwrap().unwrap();
        assert_eq!(c.name_str(), "PING");
    }

    #[test]
    fn byte_by_byte_never_panics_or_errors() {
        let full = b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let mut codec = ProtocolCodec::new();
        let mut b = BytesMut::new();
        let mut result = None;
        for &byte in full {
            b.extend_from_slice(&[byte]);
            if let Some(cmd) = codec.decode(&mut b).unwrap() {
                result = Some(cmd);
            }
        }
        let c = result.unwrap();
        assert_eq!(c.name_str(), "SET");
        assert_eq!(&c.args[0][..], b"foo");
        assert_eq!(&c.args[1][..], b"bar");
    }

    #[test]
    fn split_at_every_position() {
        let full = b"*2\r\n$3\r\nGET\r\n$5\r\nhello\r\n";

        for split in 0..=full.len() {
            let mut codec = ProtocolCodec::new();
            let mut b = BytesMut::from(&full[..split]);

            let first = codec.decode(&mut b).unwrap();

            if split < full.len() {
                assert!(first.is_none(), "split={split} should not yet complete");

                b.extend_from_slice(&full[split..]);

                let cmd = codec
                    .decode(&mut b)
                    .unwrap()
                    .expect("must complete after full input");

                assert_eq!(cmd.name_str(), "GET");
                assert_eq!(&cmd.args[0][..], b"hello");
            } else {
                let cmd = first.expect("complete input should decode immediately");

                assert_eq!(cmd.name_str(), "GET");
                assert_eq!(&cmd.args[0][..], b"hello");
            }
        }
    }

    #[test]
    fn ping() {
        let c = once("*1\r\n$4\r\nPING\r\n").unwrap().unwrap();
        assert_eq!(c.name_str(), "PING");
        assert_eq!(c.arg_len(), 0);
    }

    #[test]
    fn case_normalized() {
        assert_eq!(
            once("*1\r\n$4\r\nping\r\n").unwrap().unwrap().name_str(),
            "PING"
        );
    }

    #[test]
    fn get() {
        let c = once("*2\r\n$3\r\nGET\r\n$5\r\nhello\r\n").unwrap().unwrap();
        assert_eq!(&c.args[0][..], b"hello");
    }

    #[test]
    fn binary_safe_embedded_crlf() {
        let c = once("*2\r\n$3\r\nSET\r\n$8\r\nfoo\r\nbar\r\n")
            .unwrap()
            .unwrap();
        assert_eq!(&c.args[0][..], b"foo\r\nbar");
    }

    #[test]
    fn pipelined_commands() {
        let mut codec = ProtocolCodec::new();
        let mut b = buf("*1\r\n$4\r\nPING\r\n*1\r\n$4\r\nPING\r\n");
        assert_eq!(codec.decode(&mut b).unwrap().unwrap().name_str(), "PING");
        assert_eq!(codec.decode(&mut b).unwrap().unwrap().name_str(), "PING");
        assert!(b.is_empty());
    }

    #[test]
    fn inline_ping() {
        assert_eq!(once("PING\r\n").unwrap().unwrap().name_str(), "PING");
    }

    #[test]
    fn inline_lowercased_and_multi_space() {
        let c = once("set   foo   bar\r\n").unwrap().unwrap();
        assert_eq!(c.name_str(), "SET");
        assert_eq!(&c.args[0][..], b"foo");
        assert_eq!(&c.args[1][..], b"bar");
    }

    #[test]
    fn err_empty_array() {
        assert!(once("*0\r\n").is_err());
    }

    #[test]
    fn err_wrong_bulk_prefix() {
        assert!(once("*1\r\n+OK\r\n").is_err());
    }

    #[test]
    fn err_array_len_exceeds_limit() {
        assert!(once("*99999999999\r\n").is_err());
    }

    #[test]
    fn err_bulk_len_exceeds_limit() {
        assert!(once(&format!("*1\r\n${}\r\n", 512 * 1024 * 1024 + 1)).is_err());
    }

    #[test]
    fn err_negative_length_rejected() {
        assert!(once("*1\r\n$-5\r\n").is_err());
    }

    #[test]
    fn err_bad_crlf_after_bulk_data() {
        assert!(once("*1\r\n$3\r\nfooXX").is_err());
    }

    #[test]
    fn raw_passthrough_drains_exact_bytes_then_resumes_decoding() {
        let mut codec = ProtocolCodec::new();
        codec.begin_raw_passthrough(5);
        assert!(codec.is_in_raw_passthrough());

        let mut b = BytesMut::from(&b"REDIS*1\r\n$4\r\nPING\r\n"[..]);
        let chunk = codec.read_raw(&mut b).unwrap();
        assert_eq!(&chunk[..], b"REDIS");
        assert!(!codec.is_in_raw_passthrough());

        let cmd = codec.decode(&mut b).unwrap().unwrap();
        assert_eq!(cmd.name_str(), "PING");
    }

    #[test]
    fn raw_passthrough_across_multiple_reads() {
        let mut codec = ProtocolCodec::new();
        codec.begin_raw_passthrough(10);

        let mut b = BytesMut::from(&b"12345"[..]);
        let c1 = codec.read_raw(&mut b).unwrap();
        assert_eq!(&c1[..], b"12345");
        assert!(codec.is_in_raw_passthrough());

        b.extend_from_slice(b"67890");
        let c2 = codec.read_raw(&mut b).unwrap();
        assert_eq!(&c2[..], b"67890");
        assert!(!codec.is_in_raw_passthrough());
    }

    #[test]
    fn decode_returns_none_during_passthrough() {
        let mut codec = ProtocolCodec::new();
        codec.begin_raw_passthrough(3);
        let mut b = BytesMut::from(&b"*1\r\n$4\r\nPING\r\n"[..]);
        assert!(codec.decode(&mut b).unwrap().is_none());
    }
}
