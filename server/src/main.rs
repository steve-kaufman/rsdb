use rsdb::interface::Request;
use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let req: Request = match serde_json::from_reader(buf_reader) {
        Ok(req) => req,
        Err(e) => {
            println!("Failed to parse request: {}", e);
            return;
        }
    };
    println!("{:#?}", req);
}
