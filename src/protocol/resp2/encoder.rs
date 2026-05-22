// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use crate::protocol::reply::{CommandError, Reply};
use bytes::{BufMut, BytesMut};

pub fn encode(reply: &Reply, buf: &mut BytesMut) {
    match reply {
        Reply::Ok => buf.put_slice(b"+OK\r\n"),
        Reply::Pong => buf.put_slice(b"+PONG\r\n"),
        Reply::Null => buf.put_slice(b"$-1\r\n"),
        Reply::NullArray => buf.put_slice(b"*-1\r\n"),
        Reply::Integer(n) => {
            buf.put_u8(b':');
            encode_i64(buf, *n);
            buf.put_slice(b"\r\n");
        }
        Reply::Simple(data) => {
            buf.put_u8(b'+');
            buf.put_slice(data);
            buf.put_slice(b"\r\n");
        }
        Reply::Bulk(data) => {
            buf.put_u8(b'$');
            encode_u64(buf, data.len() as u64);
            buf.put_slice(b"\r\n");
            buf.put_slice(data);
            buf.put_slice(b"\r\n");
        }
        Reply::Array(items) => {
            buf.put_u8(b'*');
            encode_u64(buf, items.len() as u64);
            buf.put_slice(b"\r\n");
            for item in items {
                encode(item, buf);
            }
        }
        Reply::Error(err) => {
            buf.put_u8(b'-');
            encode_error(err, buf);
        }
    }
}

fn encode_u64(buf: &mut BytesMut, n: u64) {
    let mut tmp = [0u8; 20];
    let mut cursor = 20;
    let mut val = n;

    loop {
        cursor -= 1;
        tmp[cursor] = b'0' + (val % 10) as u8;
        val /= 10;
        if val == 0 {
            break;
        }
    }
    buf.put_slice(&tmp[cursor..20]);
}

fn encode_i64(buf: &mut BytesMut, n: i64) {
    if n < 0 {
        buf.put_u8(b'-');
        encode_u64(buf, n.unsigned_abs());
    } else {
        encode_u64(buf, n as u64);
    }
}

fn encode_error(err: &CommandError, buf: &mut BytesMut) {
    match err {
        CommandError::UnknownCommand => buf.put_slice(b"ERR unknown command\r\n"),
        CommandError::WrongArity => buf.put_slice(b"ERR wrong number of arguments\r\n"),
        CommandError::WrongType => {
            buf.put_slice(b"WRONGTYPE Operation against a key holding the wrong kind of value\r\n")
        }
        CommandError::OutOfRange => buf.put_slice(b"ERR value out of range\r\n"),
        CommandError::Syntax => buf.put_slice(b"ERR syntax error\r\n"),
        CommandError::Custom(msg) => {
            buf.put_slice(b"ERR ");
            buf.put_slice(msg);
            buf.put_slice(b"\r\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    fn check_encode(reply: Reply) -> String {
        let mut buf = BytesMut::new();
        encode(&reply, &mut buf);
        String::from_utf8(buf.to_vec())
            .expect("Encoded buffer should be valid UTF-8 strings in tests")
    }

    fn check_encode_bytes(reply: Reply) -> Vec<u8> {
        let mut buf = BytesMut::new();
        encode(&reply, &mut buf);
        buf.to_vec()
    }

    #[test]
    fn test_encode_ok() {
        assert_eq!(check_encode(Reply::Ok), "+OK\r\n");
    }

    #[test]
    fn test_encode_pong() {
        assert_eq!(check_encode(Reply::Pong), "+PONG\r\n");
    }

    #[test]
    fn test_encode_null() {
        // RESP2 Null Bulk String is "$-1\r\n"
        assert_eq!(check_encode(Reply::Null), "$-1\r\n");
    }

    #[test]
    fn test_encode_null_array() {
        // RESP2 Null Array is "*-1\r\n"
        assert_eq!(check_encode(Reply::NullArray), "*-1\r\n");
    }

    #[test]
    fn test_encode_simple_string() {
        assert_eq!(check_encode(Reply::simple("PING")), "+PING\r\n");
    }

    #[test]
    fn test_encode_bulk_string() {
        assert_eq!(check_encode(Reply::bulk("hello")), "$5\r\nhello\r\n");
    }

    #[test]
    fn test_encode_bulk_string_empty() {
        assert_eq!(check_encode(Reply::bulk("")), "$0\r\n\r\n");
    }

    #[test]
    fn test_encode_integer() {
        assert_eq!(check_encode(Reply::Integer(0)), ":0\r\n");
        assert_eq!(check_encode(Reply::Integer(1000)), ":1000\r\n");
        assert_eq!(check_encode(Reply::Integer(-42)), ":-42\r\n");
    }

    #[test]
    fn test_encode_empty_array() {
        assert_eq!(check_encode(Reply::Array(vec![])), "*0\r\n");
    }

    #[test]
    fn test_encode_flat_array() {
        let array = Reply::Array(vec![Reply::bulk("foo"), Reply::Integer(42), Reply::Ok]);
        assert_eq!(check_encode(array), "*3\r\n$3\r\nfoo\r\n:42\r\n+OK\r\n");
    }

    #[test]
    fn test_encode_nested_array() {
        let nested = Reply::Array(vec![
            Reply::Array(vec![Reply::bulk("bar")]),
            Reply::Integer(1),
        ]);
        assert_eq!(check_encode(nested), "*2\r\n*1\r\n$3\r\nbar\r\n:1\r\n");
    }

    #[test]
    fn test_encode_error_unknown() {
        assert_eq!(check_encode(Reply::unknown()), "-ERR unknown command\r\n");
    }

    #[test]
    fn test_encode_error_wrong_arity() {
        assert_eq!(
            check_encode(Reply::arity()),
            "-ERR wrong number of arguments\r\n"
        );
    }

    #[test]
    fn test_encode_error_wrong_type() {
        assert_eq!(
            check_encode(Reply::wrong_type()),
            "-WRONGTYPE Operation against a key holding the wrong kind of value\r\n"
        );
    }

    #[test]
    fn test_encode_error_syntax() {
        assert_eq!(check_encode(Reply::syntax()), "-ERR syntax error\r\n");
    }

    #[test]
    fn test_encode_error_out_of_range() {
        assert_eq!(
            check_encode(Reply::Error(CommandError::OutOfRange)),
            "-ERR value out of range\r\n"
        );
    }

    #[test]
    fn test_encode_error_custom() {
        assert_eq!(
            check_encode(Reply::err("something went wrong")),
            "-ERR something went wrong\r\n"
        );
    }

    #[test]
    fn test_encode_appends_to_existing_buffer() {
        let mut buf = BytesMut::from("EXISTING_DATA_");
        encode(&Reply::Ok, &mut buf);

        let result = String::from_utf8(buf.to_vec()).unwrap();
        assert_eq!(result, "EXISTING_DATA_+OK\r\n");
    }

    #[test]
    fn test_encode_binary_safety() {
        let binary_payload = vec![0x00, 0x01, 0x02, b'\r', b'\n', 0x03];
        let reply = Reply::bulk(binary_payload.clone());

        let encoded = check_encode_bytes(reply);

        let mut expected = b"$6\r\n".to_vec();
        expected.extend_from_slice(&binary_payload);
        expected.extend_from_slice(b"\r\n");

        assert_eq!(encoded, expected);
    }
}
