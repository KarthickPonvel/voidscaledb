// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::io::ErrorKind;
use std::net::SocketAddr;

use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::network::Result;
use crate::network::error::NetworkError;

pub const MIN_READ_BUFFER: usize = 64 * 1024; // 32 KB
pub const MIN_WRITE_BUFFER: usize = 64 * 1024; // 32 KB
pub const MAX_READ_BUFFER: usize = 64 * 1024 * 1024; // 32 MB
pub const MAX_WRITE_BUFFER: usize = 64 * 1024 * 1024; // 32 MB

pub struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    rbuf: BytesMut,
    wbuf: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Result<Self> {
        stream.set_nodelay(true)?;
        let addr = stream.peer_addr()?;

        Ok(Self {
            stream,
            addr,
            rbuf: BytesMut::with_capacity(MIN_READ_BUFFER),
            wbuf: BytesMut::with_capacity(MIN_WRITE_BUFFER),
        })
    }

    /// Read bytes from socket into rbuf.
    /// allocation never expands beyond MAX_READ_BUFFER under any circumstances.
    pub async fn read(&mut self) -> Result<usize> {
        let current_len = self.rbuf.len();

        if MAX_READ_BUFFER - current_len < MIN_READ_BUFFER {
            return Err(NetworkError::BufferOverflow);
        }

        let available_capacity = self.rbuf.capacity() - current_len;
        if available_capacity < MIN_READ_BUFFER {
            self.rbuf.reserve(MIN_READ_BUFFER);
        }

        match self.stream.read_buf(&mut self.rbuf).await {
            Ok(0) => Err(NetworkError::ConnectionClosed),
            Ok(n) => Ok(n), // TODO: Implement buffer(rbuf) shrink to reclaim memory.
            Err(e) => Err(match e.kind() {
                ErrorKind::ConnectionReset => NetworkError::ConnectionReset,
                ErrorKind::TimedOut => NetworkError::ConnectionTimeout,
                ErrorKind::WouldBlock => NetworkError::Io(e),
                _ => NetworkError::Io(e),
            }),
        }
    }

    /// Writes bytes from wbuf to socket.
    pub async fn write(&mut self) -> Result<()> {
        if !self.wbuf.is_empty() {
            match self.stream.write_all(&self.wbuf).await {
                Ok(_) => {
                    self.wbuf.clear();
                    // TODO: Implement buffer(wbuf) shrink to reclaim memory.
                }
                Err(e) if e.kind() == ErrorKind::WriteZero => {
                    return Err(NetworkError::ConnectionClosed);
                }
                Err(e) => return Err(NetworkError::Io(e)),
            }
        }

        Ok(())
    }

    /// Writes bytes into wbuf (Not socket).
    pub fn write_buf(&mut self, data: &[u8]) -> Result<()> {
        if self.wbuf.len() + data.len() > MAX_WRITE_BUFFER {
            return Err(NetworkError::BufferOverflow);
        }

        self.wbuf.put_slice(data);

        Ok(())
    }

    pub fn wbuf_mut(&mut self) -> &mut BytesMut {
        &mut self.wbuf
    }

    pub fn rbuf_mut(&mut self) -> &mut BytesMut {
        &mut self.rbuf
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
