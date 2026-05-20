// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

use crate::network::Result;

pub struct Listener {
    listener: TcpListener,
    addr: SocketAddr,
}

impl Listener {
    pub fn bind(addr: SocketAddr, backlog: u32) -> Result<Self> {
        let domain = if addr.is_ipv4() {
            socket2::Domain::IPV4
        } else {
            socket2::Domain::IPV6
        };

        let socket = socket2::Socket::new(domain, socket2::Type::STREAM, None)?;
        socket.set_reuse_port(true)?;
        socket.set_reuse_address(true)?;
        socket.set_nonblocking(true)?;
        socket.bind(&addr.into())?;
        socket.listen(backlog as i32)?;

        let std_listener: std::net::TcpListener = socket.into();

        let listener = TcpListener::from_std(std_listener)?;
        Ok(Self { listener, addr })
    }

    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        Ok(self.listener.accept().await?)
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
