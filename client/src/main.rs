use std::{io::Write, net::TcpStream};

use rsdb::Request;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;

    let req = Request {
        a: "Hello, world!".to_string(),
        b: 123,
        c: false,
    };
    let req_bytes = serde_json::to_vec(&req).unwrap();

    stream.write_all(&req_bytes)?;

    stream.shutdown(std::net::Shutdown::Both)
}
