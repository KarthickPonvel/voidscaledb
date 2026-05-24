// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::net::SocketAddr;

use kanal::{AsyncReceiver, AsyncSender, bounded_async};

use crate::runtime::{message::Message, worker::Worker};

pub struct Server {
    addr: SocketAddr,
    tx_channels: Vec<AsyncSender<Message>>,
    rx_channels: Vec<AsyncReceiver<Message>>,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            tx_channels: Vec::new(),
            rx_channels: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        let cores = num_cpus::get_physical();
        let (tx, rx) = Self::create_n_queues(cores, 1024);

        self.tx_channels = tx.clone();
        self.rx_channels = rx.clone();

        let handles = Self::spawn_workers(self.addr, tx, rx);

        for handle in handles {
            handle.join().ok();
        }
    }

    fn spawn_workers(
        addr: SocketAddr,
        tx: Vec<AsyncSender<Message>>,
        rx: Vec<AsyncReceiver<Message>>,
    ) -> Vec<std::thread::JoinHandle<()>> {
        let cores = rx.len();

        let mut handles = Vec::with_capacity(cores);

        for (id, rx_queue) in rx.into_iter().enumerate() {
            let thread_name = format!("worker-{}", id);
            let peers = tx.clone();

            let handle = std::thread::Builder::new()
                .name(thread_name)
                .spawn(move || {
                    Self::run_worker_lifecycle(id, addr.clone(), rx_queue, peers);
                })
                .unwrap_or_else(|_| {
                    eprintln!("thread spawn failed");
                    std::process::exit(1);
                });

            handles.push(handle);
        }
        handles
    }

    fn run_worker_lifecycle(
        id: usize,
        addr: SocketAddr,
        rx_queue: AsyncReceiver<Message>,
        tx_queues: Vec<kanal::AsyncSender<Message>>,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap_or_else(|_| {
                eprintln!("failed to build runtime-{}", id);
                std::process::exit(1);
            });

        let local = tokio::task::LocalSet::new();

        rt.block_on(async move {
            let worker = match Worker::new(id, addr, rx_queue, tx_queues) {
                Ok(worker) => worker,
                Err(_) => {
                    eprintln!("failed to create worker-{}", id);
                    std::process::exit(1);
                }
            };

            local
                .run_until(async move {
                    worker.run().await;
                })
                .await;
        });
    }

    fn create_n_queues(
        total_shards: usize,
        capacity: usize,
    ) -> (Vec<AsyncSender<Message>>, Vec<AsyncReceiver<Message>>) {
        let mut senders = Vec::with_capacity(total_shards);
        let mut receivers = Vec::with_capacity(total_shards);

        for _ in 0..total_shards {
            let (tx, rx) = bounded_async::<Message>(capacity);

            senders.push(tx);
            receivers.push(rx);
        }

        (senders, receivers)
    }
}
