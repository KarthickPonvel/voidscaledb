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

const MAX_LENGTH: usize = 512 * 1024 * 1024;

pub fn decode(
    buf: &mut BytesMut,
    state: &mut DecoderState,
    frame: &mut Option<Frame>,
) -> Result<Option<Command>> {
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
            DecoderState::ReadingArrayLength => {
                if buf[0] != b'*' {
                    return Err(ProtocolError::InvalidFrame {
                        position: 0,
                        reason: "Expected *".into(),
                    });
                }

                buf.advance(1); // consume *
                if let Some((count, consumed)) = decode_len(buf)? {
                    buf.advance(consumed);
                    *frame = Some(Frame {
                        expected_args: count,
                        current_index: 0,
                        args: SmallVec::with_capacity(count.min(16)),
                    });

                    *state = DecoderState::ReadingBulkLength
                } else {
                    return Ok(None);
                }
            }
            DecoderState::ReadingBulkLength => {
                if buf.is_empty() {
                    return Ok(None);
                }

                if buf[0] != b'$' {
                    return Err(ProtocolError::InvalidFrame {
                        position: 0,
                        reason: "Expected $".into(),
                    });
                }
                buf.advance(1); // consume $
                if let Some((len, consumed)) = decode_len(buf)? {
                    buf.advance(consumed);
                    *state = DecoderState::ReadingBulkData { remaining: len };
                } else {
                    return Ok(None);
                }
            }
            DecoderState::ReadingBulkData { remaining } => {
                let total = *remaining + 2;
                if buf.len() < total {
                    return Ok(None);
                }

                let data = buf.split_to(*remaining).freeze();
                buf.advance(2);

                let f = frame.as_mut().ok_or_else(|| ProtocolError::InvalidFrame {
                    position: 0,
                    reason: "Internal state error: Frame was lost".into(),
                })?;
                f.args.push(data);
                f.current_index += 1;

                if f.current_index == f.expected_args {
                    let final_frame = frame.take().ok_or_else(|| ProtocolError::InvalidFrame {
                        position: 0,
                        reason: "missing frame at completion".into(),
                    })?;

                    let mut args = final_frame.args;
                    let mut name = args.remove(0);

                    let mut name_vec = name.to_vec();
                    name_vec.make_ascii_uppercase();
                    name = Bytes::from(name_vec);

                    *state = DecoderState::Idle;
                    return Ok(Some(Command::new(name, args)));
                } else {
                    *state = DecoderState::ReadingBulkLength;
                }
            }
        }
    }
}

fn decode_len(buf: &[u8]) -> Result<Option<(usize, usize)>> {
    let pos = match buf.iter().position(|&b| b == b'\n') {
        Some(p) => p,
        None => return Ok(None),
    };

    if pos < 1 || buf[pos - 1] != b'\r' {
        return Err(ProtocolError::InvalidFrame {
            position: pos,
            reason: "Missing CR before LF".into(),
        });
    }

    let line = &buf[..pos - 1];

    let s = if line.starts_with(b"+") {
        &line[1..]
    } else {
        line
    };

    let val = std::str::from_utf8(s)
        .map_err(|_| ProtocolError::InvalidFrame {
            position: 0,
            reason: "Invalid UTF8".into(),
        })?
        .parse::<usize>()
        .map_err(|_| ProtocolError::InvalidFrame {
            position: 0,
            reason: "Not an integer".into(),
        })?;

    if val > MAX_LENGTH {
        return Err(ProtocolError::ConstraintViolation(format!(
            "Value {} exceeds limit {}",
            val, MAX_LENGTH
        )));
    }

    Ok(Some((val, pos + 1)))
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

    if line.last() == Some(&b'\n') {
        line.truncate(line.len() - 1);
        if line.last() == Some(&b'\r') {
            line.truncate(line.len() - 1);
        }
    } else {
        return Err(ProtocolError::InvalidFrame {
            position: end,
            reason: "Missing LF".into(),
        });
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
    use crate::protocol::{
        codec::{DecoderState, Frame},
        error::ProtocolError,
        resp2,
    };
    use bytes::BytesMut;

    fn buf(s: &str) -> BytesMut {
        BytesMut::from(s.as_bytes())
    }

    fn once(input: &str) -> crate::protocol::Result<Option<crate::protocol::command::Command>> {
        let (mut b, mut s, mut f) = (buf(input), DecoderState::Idle, None);
        resp2::decode(&mut b, &mut s, &mut f)
    }

    fn stateful(input: &str) -> (BytesMut, DecoderState, Option<Frame>) {
        let mut b = buf(input);
        let mut s = DecoderState::Idle;
        let mut f = None;
        resp2::decode(&mut b, &mut s, &mut f).unwrap();
        (b, s, f)
    }

    #[test]
    fn case_normalized_in_bulk() {
        assert_eq!(
            once("*1\r\n$4\r\nping\r\n").unwrap().unwrap().name_str(),
            "PING"
        );
    }

    #[test]
    fn ping() {
        let c = once("*1\r\n$4\r\nPING\r\n").unwrap().unwrap();
        assert_eq!(c.name_str(), "PING");
        assert_eq!(c.arg_len(), 0);
    }

    #[test]
    fn get() {
        let c = once("*2\r\n$3\r\nGET\r\n$5\r\nhello\r\n").unwrap().unwrap();
        assert_eq!(c.name_str(), "GET");
        assert_eq!(&c.args[0][..], b"hello");
    }

    #[test]
    fn set() {
        let c = once("*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n")
            .unwrap()
            .unwrap();
        assert_eq!(c.name_str(), "SET");
        assert_eq!(&c.args[0][..], b"foo");
        assert_eq!(&c.args[1][..], b"bar");
    }

    #[test]
    fn binary_safe() {
        let c = once("*2\r\n$3\r\nSET\r\n$11\r\nhello world\r\n")
            .unwrap()
            .unwrap();
        assert_eq!(&c.args[0][..], b"hello world");
    }

    #[test]
    fn embedded_crlf() {
        let mut b = BytesMut::from(&b"*2\r\n$3\r\nSET\r\n$8\r\nfoo\r\nbar\r\n"[..]);
        let c = resp2::decode(&mut b, &mut DecoderState::Idle, &mut None)
            .unwrap()
            .unwrap();
        assert_eq!(&c.args[0][..], b"foo\r\nbar");
    }

    #[test]
    fn many_args() {
        let c = once("*7\r\n$4\r\nMSET\r\n$2\r\nk1\r\n$2\r\nv1\r\n$2\r\nk2\r\n$2\r\nv2\r\n$2\r\nk3\r\n$2\r\nv3\r\n").unwrap().unwrap();
        assert_eq!(c.name_str(), "MSET");
        assert_eq!(c.arg_len(), 6);
    }

    #[test]
    fn buffer_empty_after_complete() {
        let mut b = buf("*1\r\n$4\r\nPING\r\n");
        resp2::decode(&mut b, &mut DecoderState::Idle, &mut None).unwrap();
        assert!(b.is_empty());
    }

    #[test]
    fn pipelined_second_stays_in_buf() {
        let (mut b, mut s, mut f) = stateful("*1\r\n$4\r\nPING\r\n*1\r\n$4\r\nPING\r\n");
        assert!(!b.is_empty());
        assert_eq!(
            resp2::decode(&mut b, &mut s, &mut f)
                .unwrap()
                .unwrap()
                .name_str(),
            "PING"
        );
    }

    #[test]
    fn empty() {
        assert!(once("").unwrap().is_none());
    }

    #[test]
    fn partial_array_no_crlf() {
        assert!(once("*3").unwrap().is_none());
    }

    #[test]
    fn partial_array_only_cr() {
        assert!(once("*3\r").unwrap().is_none());
    }

    #[test]
    fn array_done_no_bulk() {
        assert!(once("*1\r\n").unwrap().is_none());
    }

    #[test]
    fn bulk_header_no_data() {
        assert!(once("*1\r\n$4\r\n").unwrap().is_none());
    }

    #[test]
    fn bulk_data_truncated() {
        assert!(once("*1\r\n$4\r\nPI").unwrap().is_none());
    }

    #[test]
    fn bulk_data_no_crlf() {
        assert!(once("*1\r\n$4\r\nPING").unwrap().is_none());
    }

    #[test]
    fn resume_after_array_header() {
        let (mut b, mut s, mut f) = stateful("*2\r\n");
        b.extend_from_slice(b"$3\r\nGET\r\n$5\r\nworld\r\n");
        let c = resp2::decode(&mut b, &mut s, &mut f).unwrap().unwrap();
        assert_eq!(c.name_str(), "GET");
        assert_eq!(&c.args[0][..], b"world");
    }

    #[test]
    fn resume_after_bulk_header() {
        let (mut b, mut s, mut f) = stateful("*1\r\n$4\r\n");
        b.extend_from_slice(b"PING\r\n");
        assert_eq!(
            resp2::decode(&mut b, &mut s, &mut f)
                .unwrap()
                .unwrap()
                .name_str(),
            "PING"
        );
    }

    #[test]
    fn resume_mid_bulk_data() {
        let (mut b, mut s, mut f) = stateful("*1\r\n$4\r\nPI");
        b.extend_from_slice(b"NG\r\n");
        assert_eq!(
            resp2::decode(&mut b, &mut s, &mut f)
                .unwrap()
                .unwrap()
                .name_str(),
            "PING"
        );
    }

    #[test]
    fn resume_mid_second_arg() {
        let (mut b, mut s, mut f) = stateful("*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\n");
        b.extend_from_slice(b"bar\r\n");
        let c = resp2::decode(&mut b, &mut s, &mut f).unwrap().unwrap();
        assert_eq!(&c.args[0][..], b"foo");
        assert_eq!(&c.args[1][..], b"bar");
    }

    #[test]
    fn err_missing_cr() {
        assert!(matches!(
            once("*1\r\n$4\nPING\r\n").unwrap_err(),
            ProtocolError::InvalidFrame { .. }
        ));
    }

    #[test]
    fn err_bulk_non_integer() {
        assert!(matches!(
            once("*1\r\n$abc\r\nPING\r\n").unwrap_err(),
            ProtocolError::InvalidFrame { .. }
        ));
    }

    #[test]
    fn err_array_non_integer() {
        assert!(matches!(
            once("*abc\r\n").unwrap_err(),
            ProtocolError::InvalidFrame { .. }
        ));
    }

    #[test]
    fn err_wrong_bulk_prefix() {
        assert!(matches!(
            once("*1\r\n+OK\r\n").unwrap_err(),
            ProtocolError::InvalidFrame { .. }
        ));
    }

    #[test]
    fn err_exceeds_512mb() {
        assert!(matches!(
            once(&format!("*1\r\n${}\r\n", 512 * 1024 * 1024 + 1)).unwrap_err(),
            ProtocolError::ConstraintViolation(_)
        ));
    }

    #[test]
    fn err_utf8_in_length() {
        let mut raw = BytesMut::from("*1\r\n$".as_bytes());
        raw.extend_from_slice(&[0xFF, 0xFE, b'\r', b'\n']);
        assert!(matches!(
            resp2::decode(&mut raw, &mut DecoderState::Idle, &mut None).unwrap_err(),
            ProtocolError::InvalidFrame { .. }
        ));
    }

    #[test]
    fn inline_ping_crlf() {
        let c = once("PING\r\n").unwrap().unwrap();
        assert_eq!(c.name_str(), "PING");
        assert_eq!(c.arg_len(), 0);
    }

    #[test]
    fn inline_ping_lf() {
        let c = once("PING\n").unwrap().unwrap();
        assert_eq!(c.name_str(), "PING");
        assert_eq!(c.arg_len(), 0);
    }

    #[test]
    fn inline_set() {
        let c = once("SET foo bar\r\n").unwrap().unwrap();
        assert_eq!(c.name_str(), "SET");
        assert_eq!(&c.args[0][..], b"foo");
        assert_eq!(&c.args[1][..], b"bar");
    }

    #[test]
    fn inline_lowercased() {
        assert_eq!(once("get mykey\r\n").unwrap().unwrap().name_str(), "GET");
    }

    #[test]
    fn inline_multi_spaces() {
        let c = once("SET   foo   bar\r\n").unwrap().unwrap();
        assert_eq!(c.arg_len(), 2);
        assert_eq!(&c.args[0][..], b"foo");
    }

    #[test]
    fn inline_trailing_spaces() {
        assert_eq!(once("PING   \r\n").unwrap().unwrap().arg_len(), 0);
    }

    #[test]
    fn inline_no_newline() {
        assert!(once("PING").unwrap().is_none());
    }

    #[test]
    fn inline_partial() {
        assert!(once("SET fo").unwrap().is_none());
    }

    #[test]
    fn inline_bare_lf() {
        assert!(matches!(
            once("\n").unwrap_err(),
            ProtocolError::InvalidFrame { .. }
        ));
    }

    #[test]
    fn inline_crlf_only() {
        assert!(matches!(
            once("\r\n").unwrap_err(),
            ProtocolError::InvalidFrame { .. }
        ));
    }
}
