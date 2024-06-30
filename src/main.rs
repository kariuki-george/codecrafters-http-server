use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

use itertools::Itertools;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    // let mut handlers: HashMap<String, fn(Request) -> Response> = HashMap::new();
    // handlers.insert("/".to_string(), handler);
    // handlers.insert("/echo/{str}".to_string(), echo_handler);

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

                let mut response = runner(request);

                // Write response
                println!("{:?}", response);

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

fn runner(request: Request) -> Response {
    // Just do enough to pass the test

    let mut response = Response::new();

    if request.target.starts_with("/echo/") {
        let (_, data) = request.target.rsplit_once('/').unwrap();

        response.set_header("Content-Type".to_string(), "text/plain".to_string());
        response.set_status(200, "OK".to_string());
        response.set_body(data.into());
    }
    if request.target == "/" {
        response.set_status(200, "OK".to_string());
    }
    if request.target == "/user-agent" {
        let user_agent = request.headers.get("User-Agent").unwrap();
        response.set_header("Content-Type".to_string(), "text/plain".to_string());
        response.set_status(200, "OK".to_string());
        response.set_body(user_agent.into());
    }
    response

    // let handler = handlers.get(&request.target);

    // if let Some(handler) = handler {
    //     let response = handler(request);
    //     return response;
    // }

    // // Do a dynamic params search

    // let parts = request.target.split('/').collect::<Vec<&str>>();
    // if parts.len() < 2 {
    //     // Path not found

    //     let mut res = Response::new();
    //     res.set_status(404, "Not Found".to_string());
    //     return res;
    // }

    // Try matching dynamic paths to parts
    // /path/{dyn}
    // /echo/123
    // dyn = 123

    // 'outer: for key in handlers.keys() {
    //     let key_parts = key.split('/').collect_vec();
    //     if key_parts.len() != parts.len() {
    //         continue;
    //     }

    //     let length = key_parts.len();

    //     for (index, part) in key_parts.iter().enumerate() {
    //         if !part.starts_with('{') {
    //             if *part != parts[index] {
    //                 continue 'outer;
    //             }

    //         }

    //         request.query_param.insert(k, v)

    //     }
    // }
    // let mut res = Response::new();
    // res.set_status(404, "Not Found".to_string());
    // return res;
}

// fn handler(input: Request) -> Response {}

// fn echo_handler(input: Request) -> Response {}

#[derive(Debug)]

struct Request {
    http_method: HTTPMethod,
    target: String,
    http_version: String,
    headers: HashMap<String, String>,
    body: Option<String>,
    query_param: HashMap<String, String>,
}

impl Request {
    fn new(req: Vec<String>) -> Result<Request, String> {
        let mut request = Request {
            http_method: HTTPMethod::Get,
            body: None,
            headers: HashMap::new(),
            http_version: String::new(),
            target: String::new(),
            query_param: HashMap::new(),
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

trait ResponseValue {}

#[derive(Clone, Debug)]
struct Response {
    status: u16,
    body: Option<String>,
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

    fn set_body(&mut self, body: String) {
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

        // Body
        if let Some(body) = self.body.clone() {
            response_string.push_str(&body);
        }

        // Full response string
        response_string
    }
    fn into_bytes(&mut self) -> Vec<u8> {
        self.as_string().into_bytes()
    }
}
