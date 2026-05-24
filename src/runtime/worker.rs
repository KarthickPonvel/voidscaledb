// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::{cell::RefCell, net::SocketAddr, rc::Rc};

use tokio::task::spawn_local;

use crate::{
    engine::shard::ShardEngine,
    network::{connection::Connection, listener::Listener},
    runtime::{
        affinity::pin_thread_to_core,
        error::{Result, RuntimeError},
        handler::{handle_conn, handle_inter_shard_message},
        message::Message,
        resolver::ShardResolver,
    },
};

pub struct Worker {
    id: usize,
    listener: Listener,
    rx_queue: kanal::AsyncReceiver<Message>,
    engine: Rc<RefCell<ShardEngine>>,
    resolver: Rc<ShardResolver>,
}

impl Worker {
    pub fn new(
        id: usize,
        addr: SocketAddr,
        rx_queue: kanal::AsyncReceiver<Message>,
        peers: Vec<kanal::AsyncSender<Message>>,
    ) -> Result<Self> {
        let listener = match Listener::bind(addr, 128) {
            Ok(listener) => listener,
            Err(_) => {
                return Err(RuntimeError::WorkerStartFailed {
                    id,
                    reason: "failed to bind listener socket".into(),
                });
            }
        };

        pin_thread_to_core(id)?;

        let engine = Rc::new(RefCell::new(ShardEngine::new()));
        let resolver = Rc::new(ShardResolver::new(id, peers));
        Ok(Self {
            id,
            listener,
            rx_queue,
            engine,
            resolver,
        })
    }

    pub async fn run(&self) {
        println!("Worker {} started running", self.id);
        loop {
            tokio::select! {
                biased;


                Ok(msg) = self.rx_queue.recv() => {
                    let engine = self.engine.clone();

                    spawn_local(async move {
                        handle_inter_shard_message(msg, engine).await;
                    });
                }

                Ok((stream, _addr)) = self.listener.accept() => {
                    let conn = match Connection::new(stream) {
                        Ok(conn) => conn,
                        Err(_) => continue,
                    };

                    let resolver = self.resolver.clone();
                    let engine = self.engine.clone();

                    spawn_local(async move {
                        handle_conn(conn, resolver, engine).await
                    });
                }
            }
        }
    }
}
