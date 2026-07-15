// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use std::net::SocketAddr;

use crate::runtime::runtime::Runtime;

pub struct Server {
    _addr: SocketAddr,
    runtime: Runtime,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            _addr: addr,
            runtime: Runtime::new(addr),
        }
    }

    pub fn start(self) {
        self.runtime.run();
    }
}
