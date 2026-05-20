// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::net::SocketAddr;

use bytes::Buf;

use crate::network::{Result, connection::Connection, error::NetworkError, listener::Listener};

pub struct Server {
    listener: Listener,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Result<Self> {
        let listener = match Listener::bind(addr, 128) {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Error creating a socket {:?}", e);

                // If no server listener(socket) no will listen to client so stop executing program.
                panic!("Failed to create listening socket")
            }
        };

        Ok(Self { listener })
    }

    pub async fn start(&mut self) {
        println!("Server start running on {:?}", self.listener.addr());
        loop {
            let (stream, client_addr) = match self.listener.accept().await {
                Ok((stream, addr)) => (stream, addr),
                Err(e) => {
                    eprintln!("Client connection error: {:?}", e);
                    return;
                }
            };

            println!("Client connected from {:?}", client_addr);

            tokio::spawn(async move {
                let mut conn = match Connection::new(stream) {
                    Ok(c) => c,

                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        return;
                    }
                };

                loop {
                    let n = match conn.read().await {
                        Ok(n) => n,
                        Err(NetworkError::ConnectionClosed) => {
                            eprintln!("Connection closed {:?}", conn.addr());
                            return;
                        }
                        Err(_) => {
                            eprintln!("Error read from socket");
                            return;
                        }
                    };

                    let data = conn.rbuf_mut().to_vec();

                    if let Err(_) = conn.write_buf(&data) {
                        eprintln!("Error write in buffer")
                    };

                    if let Err(_) = conn.write().await {
                        eprintln!("Error write to socket")
                    };

                    conn.rbuf_mut().advance(n);
                }
            });
        }
    }
}
