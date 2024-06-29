use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

use itertools::Itertools;
use serde_json::Value;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let mut handlers = HashMap::new();
    handlers.insert("/".to_string(), random_handler);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // Read request
                let mut reader = BufReader::new(&stream);

                // Split lines using a space
                let mut http_parts = vec![];
                let lines = reader
                    .by_ref()
                    .lines()
                    .map(|x| x.unwrap())
                    .take_while(|line| !line.is_empty())
                    .collect_vec();

                for line in lines.clone() {
                    http_parts.push(line);
                    http_parts.push(" ".to_string());
                }

                // Remove the last delimiter
                http_parts.pop();

                let mut request = Request::new(http_parts).unwrap();

                // Read the body if content-length is available
                if let Some(length) = request.headers.get("Content-Length") {
                    let length = length.parse::<usize>().unwrap();

                    let mut buf = vec![0; length];

                    reader.read_exact(&mut buf).unwrap();

                    let data = String::from_utf8(buf).unwrap();
                    request.body = Some(data)
                }

                println!("{:?}", request);

                // Handle request

                // Get handler

                let handler = handlers.get(&request.target);

                // Write response
                let mut response = Response::new();
                if handler.is_none() {
                    response.set_status(400, "Not Found".to_string());
                }

                let response = response.into_bytes();

                stream.write_all(&response).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn random_handler() {}

#[derive(Debug)]

struct Request {
    http_method: HTTPMethod,
    target: String,
    http_version: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Request {
    fn new(req: Vec<String>) -> Result<Request, String> {
        let mut request = Request {
            http_method: HTTPMethod::Get,
            body: None,
            headers: HashMap::new(),
            http_version: String::new(),
            target: String::new(),
        };

        // Parse input request string
        let mut req = req.iter();

        // Parse request line

        // Should have three parts

        let mut parts = req
            .next()
            .expect("Error: Invalid http request")
            .split(' ')
            .collect::<Vec<&str>>();

        if parts.len() != 3 {
            return Err("Error parsing the http request".to_string());
        }

        request.http_version = parts.pop().unwrap().to_string();
        request.target = parts.pop().unwrap().to_string();

        // HTTP METHODs
        request.http_method = match parts.pop().unwrap() {
            "GET" => HTTPMethod::Get,
            "POST" => HTTPMethod::Post,
            _ => return Err("Error: HTTP method passed not supported".to_string()),
        };
        // Parse the headers
        if req.next().is_none() {
            return Ok(request);
        };

        for part in req {
            if part == " " {
                continue;
            }
            if part.is_empty() {
                break;
            }
            // Parse header
            let (key, value) = part
                .split_once(": ")
                .expect("Error: failed to parse http headers");

            request.headers.insert(key.to_owned(), value.to_owned());
        }

        Ok(request)
    }
}

#[derive(Debug)]
enum HTTPMethod {
    Get,
    Post,
}

#[derive(Clone)]
struct Response {
    status: u16,
    body: Option<Value>,
    status_reason: String,
    headers: HashMap<String, serde_json::Value>,
}

impl Response {
    fn new() -> Response {
        Response {
            body: None,
            status: 200,
            status_reason: "OK".to_string(),
            headers: HashMap::new(),
        }
    }

    fn set_status(&mut self, status: u16, status_reason: String) -> Self {
        self.status = status;
        self.status_reason = status_reason;
        self.to_owned()
    }

    fn set_body(&mut self, body: Value) -> Self {
        self.body = Some(body);
        self.to_owned()
    }
    fn as_string(&mut self) -> String {
        // Status Line
        let mut response_string = String::from("HTTP/1.1");

        if self.status == 0 {
            self.status = 200
        }

        response_string.push_str(&format!(" {}", self.status));

        response_string.push_str(&format!(" {}", self.status_reason));

        response_string.push_str("\r\n");

        // Headers
        response_string.push_str("\r\n");

        // Body

        // Full response string
        response_string
    }
    fn into_bytes(&mut self) -> Vec<u8> {
        self.as_string().into_bytes()
    }
}
