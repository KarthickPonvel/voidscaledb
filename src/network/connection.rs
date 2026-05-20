// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::io::ErrorKind;
use std::net::SocketAddr;

use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::network::Result;
use crate::network::error::NetworkError;

pub const MIN_READ_BUFFER: usize = 32 * 1024; // 32 KB
pub const MIN_WRITE_BUFFER: usize = 32 * 1024; // 32 KB
pub const MAX_READ_BUFFER: usize = 32 * 1024 * 1024; // 32 MB
pub const MAX_WRITE_BUFFER: usize = 32 * 1024 * 1024; // 32 MB

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
    pub async fn read(&mut self) -> Result<usize> {
        if self.rbuf.len() >= MAX_READ_BUFFER {
            return Err(NetworkError::BufferOverflow);
        }

        let remaining = MAX_READ_BUFFER - self.rbuf.len();
        self.rbuf.reserve(remaining.min(MIN_READ_BUFFER));

        let chunk = self.rbuf.chunk_mut();

        // SAFETY: chunk_mut() gives uninitialized tail of rbuf.
        // advance_mut(n) called only with exact byte count from read().
        let buf = unsafe { std::slice::from_raw_parts_mut(chunk.as_mut_ptr(), chunk.len()) };

        match self.stream.read(buf).await {
            Ok(0) => return Err(NetworkError::ConnectionClosed),
            Ok(n) => {
                unsafe {
                    self.rbuf.advance_mut(n);
                }

                Ok(n)
            }
            Err(e) => {
                return Err(match e.kind() {
                    ErrorKind::ConnectionReset => NetworkError::ConnectionReset,
                    ErrorKind::TimedOut => NetworkError::ConnectionTimeout,
                    ErrorKind::WouldBlock => NetworkError::Io(e),
                    _ => NetworkError::Io(e),
                });
            }
        }
    }

    /// Writes bytes from wbuf to socket.
    pub async fn write(&mut self) -> Result<()> {
        while !self.wbuf.is_empty() {
            let n = self.stream.write(&self.wbuf).await?;

            if n == 0 {
                return Err(NetworkError::ConnectionClosed);
            }

            self.wbuf.advance(n);
        }
        Ok(())
    }

    /// Writes bytes into wbuf(Not socket).
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
