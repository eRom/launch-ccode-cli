use lcc::proxy::health::is_alive;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

/// Mini HTTP server qui répond 200 sur GET /health/liveness.
fn spawn_mock_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming().take(5) {
            if let Ok(mut s) = stream {
                use std::io::{Read, Write};
                let mut buf = [0u8; 1024];
                let _ = s.read(&mut buf);
                let _ = s.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 21\r\n\r\n{\"status\":\"healthy\"}\n",
                );
            }
        }
    });
    // Donne un peu de temps au listener
    thread::sleep(Duration::from_millis(50));
    port
}

#[test]
fn alive_when_server_responds_200() {
    let port = spawn_mock_server();
    assert!(is_alive(port, Duration::from_secs(1)));
}

#[test]
fn dead_when_no_server() {
    // Port très probablement libre
    assert!(!is_alive(59999, Duration::from_millis(200)));
}
