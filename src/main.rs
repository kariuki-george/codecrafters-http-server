use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

use flate2::{write::GzEncoder, Compression};
use itertools::Itertools;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        // For concurrency, spawn a thread to handle a connection
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

                let mut response = runner(request);

                // Write response
                println!("{:?}", response);

                let response = response.as_bytes();

                stream.write_all(&response).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn runner(request: Request) -> Response {
    // Just do enough to pass the tests instead of building or using a full blown router.

    let mut response = Response::new();

    if request.target.starts_with("/echo/") {
        let (_, data) = request.target.rsplit_once('/').unwrap();

        response.set_header("Content-Type".to_string(), "text/plain".to_string());
        response.set_status(200, "OK".to_string());

        response.set_body(data.as_bytes().into());
    }
    if request.target == "/" {
        response.set_status(200, "OK".to_string());
    }
    if request.target == "/user-agent" {
        let user_agent = request.headers.get("User-Agent").unwrap();
        response.set_header("Content-Type".to_string(), "text/plain".to_string());
        response.set_status(200, "OK".to_string());
        response.set_body(user_agent.as_bytes().into());
    }
    if request.target.starts_with("/files/") {
        let (_, file) = request.target.rsplit_once('/').unwrap();
        let args = std::env::args();
        let mut path = String::new();
        // Could write cleaner code
        let args = args.collect_vec();
        for (index, arg) in args.clone().iter().enumerate() {
            if arg == "--directory" {
                if args.len() == index + 1 {
                    // Handle error gracefully
                    panic!()
                }
                path.clone_from(&args[index + 1]);
            }
        }

        //  Check if the file exists in the provided directory
        path.push_str(file);

        // Get request
        match request.http_method {
            HTTPMethod::Get => {
                let mut file = match std::fs::OpenOptions::new().read(true).open(path) {
                    Ok(file) => file,
                    Err(_) => return response,
                };

                let mut contents = String::new();

                if file.read_to_string(&mut contents).is_err() {
                    return response;
                }
                response.set_status(200, "OK".to_string());
                response.set_header(
                    "Content-Type".to_string(),
                    "application/octet-stream".to_string(),
                );
                response.set_body(contents.as_bytes().into());
            }
            HTTPMethod::Post => {
                let data = request.body.unwrap();

                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(path)
                    .unwrap();

                file.write_all(data.as_bytes()).unwrap();
                file.flush().unwrap();

                response.set_status(201, "Created".to_string());
            }
        }
    }

    // Compress the body

    if let Some(encodings) = request.headers.get("Accept-Encoding") {
        // If not supported
        let supported_encodings = ["gzip"];

        for encoding in encodings.split(',') {
            let encoding = encoding.trim();
            if supported_encodings.contains(&encoding) {
                // Use it
                if encoding == "gzip" {
                    response.set_header("Content-Encoding".to_string(), "gzip".to_string());
                    if let Some(body) = &response.body {
                        let mut e = GzEncoder::new(Vec::new(), Compression::default());

                        e.write_all(body).unwrap();
                        let compressed_bytes = e.finish().unwrap();

                        response.set_body(compressed_bytes);
                    }
                }

                break;
            }
        }
    }

    response
}

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

#[derive(Clone, Debug)]
struct Response {
    status: u16,
    body: Option<Vec<u8>>,
    status_reason: String,
    headers: HashMap<String, String>,
}

impl Response {
    fn new() -> Response {
        Response {
            body: None,
            status: 404,
            status_reason: "Not Found".to_string(),
            headers: HashMap::new(),
        }
    }

    fn set_status(&mut self, status: u16, status_reason: String) {
        self.status = status;
        self.status_reason = status_reason;
    }

    fn set_body(&mut self, body: Vec<u8>) {
        self.set_header("Content-Length".to_string(), format!("{}", body.len()));
        self.body = Some(body);
    }

    fn set_header(&mut self, name: String, value: String) {
        self.headers.insert(name, value);
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
        for (name, value) in self.headers.clone() {
            let header = format!("{}: {}\r\n", name, value);

            response_string.push_str(&header);
        }
        response_string.push_str("\r\n");

        // Full response string
        response_string
    }
    fn as_bytes(&mut self) -> Vec<u8> {
        let mut response_bytes = self.as_string().into_bytes();
        // Add the body
        // Body
        if let Some(body) = self.body.clone() {
            response_bytes = [response_bytes, body].concat();
        }
        response_bytes
    }
}
