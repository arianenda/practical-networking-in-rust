use std::{
    collections::HashMap, fmt, fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, sync::Arc
};

use practical_networking_in_rust::ThreadPool;

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
struct Request {
    method: String,
    path: String
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum PageContent {
    Home, 
    About,
    Contact
}

impl PageContent {
    fn get_file_path(&self) -> String {
        match self {
            PageContent::Home => "src/static/index.html".to_string(),
            PageContent::About => "src/static/about.html".to_string(),
            PageContent::Contact => "src/static/contact.html".to_string()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RequestHandler {
    routes: HashMap<String, PageContent> 
}

impl RequestHandler {
    fn new() -> Self {
        let mut routes: HashMap<String, PageContent> = HashMap::new();
        routes.insert("/".to_string(), PageContent::Home);
        routes.insert("/about".to_string(), PageContent::About);
        routes.insert("/contact".to_string(), PageContent::Contact);

        RequestHandler { routes }
    }

    fn handle_request(&self, request: &Request) -> Response {
        let method = &request.method;
        if method == "GET" {
            if let Some(file_content) = self.routes.get(&request.path) {
                match fs::read_to_string(file_content.get_file_path()) {
                    Ok(content) => {
                        return Response::new(HttpStatus::Ok, content)
                    }
                    Err(_) => {
                        return Response::new(HttpStatus::InternalServerError, "Internal Server Error".to_string())
                    }
                }
            }
        }

        Response::new(HttpStatus::NotFound, fs::read_to_string("src/static/404.html").unwrap())
    }
}

fn parse_request(stream_buffer: BufReader<&TcpStream>) -> Request {
    let first_line = stream_buffer.lines().next().unwrap().unwrap();
    let mut tokens = first_line.split_whitespace();
    let method = tokens.next().unwrap().to_string();
    let path = tokens.next().unwrap().to_string();

    Request {
        method,
        path
    }
}

fn handle_incoming_connection(mut stream: TcpStream, req_handler: Arc<RequestHandler>) {
    let stream_buf = BufReader::new(&stream);
    let request = parse_request(stream_buf);

    let response = req_handler.handle_request(&request);
    let resp_as_string = format!("{}", response);
    let resp_as_bytes = resp_as_string.as_bytes();

    stream.write_all(resp_as_bytes).unwrap();

    println!("=======[Request triggered]========");
    println!("Http Requests: {:#?}", request);
    println!("Http Response: {:#?}", resp_as_string);
    println!("==================================");
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    // limit thread by 4
    let pool = ThreadPool::new(4);

    let request_handler = Arc::new(RequestHandler::new());
    let mut conn_incoming = listener.incoming();


    while let Some(Ok(stream)) = conn_incoming.next() {
        let req_handle_clone = Arc::clone(&request_handler);
        pool.execute(move || {
            handle_incoming_connection(stream, req_handle_clone);
        });
    }
}


