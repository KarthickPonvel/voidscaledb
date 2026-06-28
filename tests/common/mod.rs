use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

pub const PORT: u16 = 9379;

pub struct ServerHandle {
    child: Child,
}

impl ServerHandle {
    pub fn start() -> Self {
        let child = Command::new("cargo")
            .args(["run", "--", "--port", &PORT.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn DB server process");

        let handle = ServerHandle { child };
        handle.wait_until_ready();
        handle
    }

    fn wait_until_ready(&self) {
        let addr = format!("127.0.0.1:{}", PORT);
        for _ in 0..100 {
            if TcpStream::connect(&addr).is_ok() {
                println!("[common] DB server ready on port {}", PORT);
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }
        panic!(
            "[common] DB server on port {} never became ready after 10s",
            PORT
        );
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        println!("[common] Shutting down DB server");
        self.child.kill().ok();
        self.child.wait().ok();
    }
}

static SERVER: OnceLock<ServerHandle> = OnceLock::new();

pub fn setup_server() {
    SERVER.get_or_init(|| ServerHandle::start());
}

pub fn connect() -> redis::Connection {
    setup_server();

    let url = format!("redis://127.0.0.1:{}/", PORT);
    let client = redis::Client::open(url).expect("failed to create redis client");

    client
        .get_connection()
        .expect("failed to connect to DB — is server running on port 9379?")
}
