use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let mut conn_incoming = listener.incoming();

    while let Some(Ok(stream)) = conn_incoming.next() {
        handle_incoming_connection(stream);
    }
}

fn handle_incoming_connection(stream: TcpStream) {
    let stream_buf = BufReader::new(&stream);
    let http_req: Vec<_> = stream_buf
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Http Requests: {:#?}", http_req);
}
