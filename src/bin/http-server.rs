use std::{
    fmt, fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum HttpStatus {
    Ok,
    NotFound,
    InternalServerError,
}

impl HttpStatus {
    fn status_code(&self) -> u16 {
        match self {
            HttpStatus::Ok => 200,
            HttpStatus::NotFound => 404,
            HttpStatus::InternalServerError => 500,
        }
    }

    fn status_info(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::NotFound => "NOT FOUND",
            HttpStatus::InternalServerError => "INTERNAL SERVER ERROR",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Response {
    http_status: HttpStatus,
    contents: String,
    contents_length: usize,
}

impl Response {
    fn new(http_status: HttpStatus, contents: String) -> Self {
        let contents_length = contents.len();
        Response {
            http_status,
            contents,
            contents_length,
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTTP/1.1 {} {}\r\nContent-Lenght: {}\r\n\r\n{}",
            self.http_status.status_code(),
            self.http_status.status_info(),
            self.contents_length,
            self.contents
        )
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let mut conn_incoming = listener.incoming();

    while let Some(Ok(stream)) = conn_incoming.next() {
        handle_incoming_connection(stream);
    }
}

fn handle_incoming_connection(mut stream: TcpStream) {
    let stream_buf = BufReader::new(&stream);
    let http_req = stream_buf.lines().next().unwrap().unwrap();

    let response: Response = match http_req.as_str() {
        "GET / HTTP/1.1" => {
            let content = fs::read_to_string("src/static/index.html");
            match content {
                Ok(content) => Response::new(HttpStatus::Ok, content),
                Err(_) => Response::new(
                    HttpStatus::InternalServerError,
                    "Internal Server Error 500 {}".to_string(),
                ),
            }
        }
        _ => Response::new(
            HttpStatus::NotFound,
            fs::read_to_string("src/static/404.html").unwrap(),
        ),
    };

    let resp_as_string = format!("{}", response);
    let resp_as_bytes = resp_as_string.as_bytes();

    stream.write_all(resp_as_bytes).unwrap();

    println!("=======[Request triggered]========");
    println!("Http Requests: {:#?}", http_req);
    println!("Http Response: {:#?}", resp_as_string);
    println!("==================================");
}
