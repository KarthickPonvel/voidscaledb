// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::{net::SocketAddr, rc::Rc};

use crate::{
    engine::{
        coordinator::Coordinator,
        message::{MessageRx, MessageTx},
    },
    network::{client::Client, listener::Listener},
    runtime::error::{Result, RuntimeError},
};

pub struct Worker {
    id: usize,
    listener: Listener,
    coordinator: Rc<Coordinator>,
}

impl Worker {
    pub fn new(id: usize, addr: SocketAddr, rx: MessageRx, peers: Vec<MessageTx>) -> Result<Self> {
        let listener = match Listener::bind(addr, 128) {
            Ok(listener) => listener,
            Err(_) => {
                return Err(RuntimeError::WorkerStartFailed {
                    id,
                    reason: "failed to bind listener socket".into(),
                });
            }
        };
        let coordinator = Rc::new(Coordinator::new(id, rx, peers));
        Ok(Worker {
            id,
            listener,
            coordinator,
        })
    }

    pub async fn run(&mut self) {
        let coordinator_for_bus = self.coordinator.clone();
        tokio::task::spawn_local(async move {
            coordinator_for_bus.run().await;
        });
        loop {
            match self.listener.accept().await {
                Ok((stream, _addr)) => {
                    let coordinator = self.coordinator.clone();
                    let mut client = Client::new(stream, coordinator).unwrap();
                    // self.client.push(client);
                    tokio::task::spawn_local(async move {
                        client.run().await;
                    });
                }
                Err(_) => {}
            }
        }
    }
}
