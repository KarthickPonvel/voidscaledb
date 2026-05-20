// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use voidscale::network::server::Server;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut server = Server::new("127.0.0.1:9379".parse().unwrap()).unwrap();
        server.start().await;
    })
}
