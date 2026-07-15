// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::{net::SocketAddr, thread::JoinHandle};

use crate::{
    engine::message::{Message, MessageRx, MessageTx},
    runtime::{
        error::{RuntimeError, RuntimeResult},
        worker::Worker,
    },
};

pub struct Runtime {
    worker_count: usize,
    workers: Vec<WorkerHandle>,
    addr: SocketAddr,
}

struct WorkerHandle {
    _id: usize,
    handle: JoinHandle<()>,
}

impl Runtime {
    pub fn new(addr: SocketAddr) -> Self {
        // let worker_count = num_cpus::get_physical(); // Multi shard
        let worker_count = 1; // Single shard
        let workers = Vec::new();
        Self {
            worker_count,
            workers,
            addr,
        }
    }

    pub fn run(mut self) {
        self.workers = Self::spawn_workers(self.worker_count, self.addr);

        for worker in self.workers {
            worker.handle.join().unwrap();
        }
    }

    fn spawn_workers(count: usize, addr: SocketAddr) -> Vec<WorkerHandle> {
        let (txs, mut rxs) = Self::create_worker_channels(count, 1024);
        let mut handles = Vec::new();

        for id in 0..count {
            let thread_name = format!("worker-{}", id);
            let rx = rxs.remove(0);
            let peers = txs.clone();

            let handle = std::thread::Builder::new()
                .name(thread_name)
                .spawn(move || {
                    Runtime::pin_thread_to_core(id).unwrap();
                    Runtime::run_worker(id, addr.clone(), rx, peers);
                })
                .unwrap_or_else(|_| {
                    eprintln!("thread spawn failed");
                    std::process::exit(1);
                });

            handles.push(WorkerHandle::new(id, handle));
        }
        return handles;
    }

    fn run_worker(id: usize, addr: SocketAddr, rx: MessageRx, peers: Vec<MessageTx>) {
        let runtime = Self::build_tokio_runtime(id);
        let local = tokio::task::LocalSet::new();

        runtime.block_on(local.run_until(async move {
            let mut worker = match Worker::new(id, addr, rx, peers) {
                Ok(w) => w,
                Err(_) => {
                    eprintln!("failed to create worker-{}", id);
                    std::process::exit(1);
                }
            };

            worker.run().await
        }))
    }

    fn build_tokio_runtime(id: usize) -> tokio::runtime::Runtime {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap_or_else(|_| {
                eprintln!("failed to build runtime - {}", id);
                std::process::exit(1);
            });

        runtime
    }

    fn pin_thread_to_core(core_id: usize) -> RuntimeResult<()> {
        match core_affinity::get_core_ids() {
            Some(ids) if !ids.is_empty() => {
                let core = ids[core_id % ids.len()];
                core_affinity::set_for_current(core);
                Ok(())
            }
            _ => Err(RuntimeError::ThreadPinFailed {
                id: core_id,
                core: core_id,
            }),
        }
    }

    fn create_worker_channels(
        worker_count: usize,
        capacity: usize,
    ) -> (Vec<MessageTx>, Vec<MessageRx>) {
        let mut txs = Vec::with_capacity(worker_count);
        let mut rxs = Vec::with_capacity(worker_count);

        for _ in 0..worker_count {
            let (tx, rx) = crossfire::mpsc::bounded_async::<Message>(capacity);
            txs.push(tx);
            rxs.push(rx);
        }

        (txs, rxs)
    }
}

impl WorkerHandle {
    pub fn new(id: usize, handle: JoinHandle<()>) -> Self {
        Self { _id: id, handle }
    }
}
