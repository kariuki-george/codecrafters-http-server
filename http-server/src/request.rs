use std::collections::HashMap;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
};

#[derive(Debug, Clone)]

pub struct Request {
    pub http_method: HTTPMethod,
    pub target: String,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub path_variables: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

impl Request {
    pub async fn new(stream: &mut TcpStream) -> Result<Request, String> {
        // Read request
        let reader = &mut BufReader::new(stream);

        // Split lines using a space
        let mut http_parts = vec![];

        let lines = &mut reader.lines();

        while let Some(line) = lines.next_line().await.unwrap() {
            if line.is_empty() {
                break;
            }

            http_parts.push(line.clone());
            http_parts.push(" ".to_string());
        }

        // Remove the last delimiter
        http_parts.pop();
        let mut request = Request {
            http_method: HTTPMethod::Get,
            body: None,
            headers: HashMap::new(),
            http_version: String::new(),
            target: String::new(),
            path_variables: HashMap::new(),
            query_params: HashMap::new(),
        };

        // Parse input request string
        let mut req = http_parts.iter();

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
        // Parse target and query params if any
        let target = parts.pop().unwrap().to_string();

        if target.contains('?') {
            let (target, q_params) = target.split_once('?').unwrap();
            request.target = target.to_string();

            for q_param in q_params.split('&') {
                let q = q_param.split_once('=');

                if q.is_none() {
                    return Err("Invalid query params passed".to_string());
                }

                let (k, v) = q.unwrap();

                request.query_params.insert(k.to_string(), v.to_string());
            }
        } else {
            request.target = target
        }

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

        // Read the body if content-length is available
        if let Some(length) = request.headers.get("Content-Length") {
            let length = length.parse::<usize>().unwrap();

            let mut buf = vec![0; length];

            reader.read_exact(&mut buf).await.unwrap();

            let data = String::from_utf8(buf).unwrap();
            request.body = Some(data)
        }

        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HTTPMethod {
    Get,
    Post,
}
