// Copyright (c) 2026-present, Karthick P.
// Licensed under the Apache License, Version 2.0.

use mimalloc::MiMalloc;
use voidscale::server::Server;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const BANNER: &str = r#"
██╗   ██╗ ██████╗ ██╗██████╗ ███████╗ ██████╗ █████╗ ██╗     ███████╗
██║   ██║██╔═══██╗██║██╔══██╗██╔════╝██╔════╝██╔══██╗██║     ██╔════╝
██║   ██║██║   ██║██║██║  ██║███████╗██║     ███████║██║     █████╗  
╚██╗ ██╔╝██║   ██║██║██║  ██║╚════██║██║     ██╔══██║██║     ██╔══╝  
 ╚████╔╝ ╚██████╔╝██║██████╔╝███████║╚██████╗██║  ██║███████╗███████╗
  ╚═══╝   ╚═════╝ ╚═╝╚═════╝ ╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝
                        D B   ·   v0.1.0
"#;

fn print_banner() {
    println!("{}", BANNER);
    println!("  VoidscaleDB starting on 127.0.0.1:9379");
    println!();
}

fn main() {
    print_banner();

    let server = Server::new("127.0.0.1:9379".parse().unwrap());
    server.start();
}
