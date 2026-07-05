// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::rc::Rc;

use tokio::net::TcpStream;

use crate::network::Result;
use crate::{
    engine::coordinator::Coordinator,
    network::connection::Connection,
    protocol::{
        codec::ProtocolCodec,
        reply::{CommandError, Reply},
    },
};

pub struct Client {
    conn: Connection,
    codec: ProtocolCodec,
    coordinator: Rc<Coordinator>,
}

impl Client {
    pub fn new(stream: TcpStream, coordinator: Rc<Coordinator>) -> Result<Self> {
        let conn = Connection::new(stream)?;
        Ok(Self {
            conn,
            codec: ProtocolCodec::new(),
            coordinator,
        })
    }

    pub async fn run(&mut self) {
        loop {
            if self.conn.read().await.is_err() {
                return;
            }
            loop {
                let cmd = match self.codec.decode(self.conn.rbuf_mut()) {
                    Ok(Some(cmd)) => cmd,
                    Ok(None) => break,
                    Err(_) => {
                        let err_reply = Reply::Error(CommandError::Syntax);
                        self.codec.encode(self.conn.wbuf_mut(), err_reply);
                        return;
                    }
                };
                let reply = self.coordinator.execute(cmd).await;
                self.codec.encode(self.conn.wbuf_mut(), reply);
            }
            if self.conn.write().await.is_err() {
                return;
            }
        }
    }
}
