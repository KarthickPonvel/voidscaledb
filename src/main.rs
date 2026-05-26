// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use voidscale::network::server::Server;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    let mut server = Server::new("127.0.0.1:9379".parse().unwrap());
    server.start();
}
