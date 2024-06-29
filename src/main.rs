use std::{io::Write, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut response = Response::new().set_status(200, None);
                let response = response.into_bytes();

                stream.write_all(&response).unwrap();
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

#[derive(Clone)]
struct Response {
    status: u16,
    body: Option<String>,
    status_reason: Option<String>,
}

impl Response {
    fn new() -> Response {
        Response {
            body: None,
            status: 0,
            status_reason: None,
        }
    }

    fn set_status(&mut self, status: u16, status_reason: Option<String>) -> Self {
        self.status = status;
        self.status_reason = status_reason;
        self.to_owned()
    }

    fn set_body(&mut self, body: String) -> Self {
        self.body = Some(body);
        self.to_owned()
    }
    fn as_string(&mut self) -> String {
        // Status Line
        let mut response_string = String::from("HTTP/1.1 ");

        if self.status == 0 {
            self.status = 200
        }

        response_string.push_str(&format!("{} ", self.status));

        if let Some(status_reason) = self.status_reason.clone() {
            response_string.push_str(&status_reason.to_string());
        };
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
