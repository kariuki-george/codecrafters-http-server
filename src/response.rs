use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Response {
    status: u16,
    pub body: Option<Vec<u8>>,
    status_reason: String,
    headers: HashMap<String, String>,
}

impl Response {
    pub fn new() -> Response {
        Response {
            body: None,
            status: 404,
            status_reason: "Not Found".to_string(),
            headers: HashMap::new(),
        }
    }

    pub fn set_status(&mut self, status: u16, status_reason: String) {
        self.status = status;
        self.status_reason = status_reason;
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.set_header("Content-Length".to_string(), format!("{}", body.len()));
        self.body = Some(body);
    }

    pub fn set_header(&mut self, name: String, value: String) {
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
    pub fn as_bytes(&mut self) -> Vec<u8> {
        let mut response_bytes = self.as_string().into_bytes();
        // Add the body
        // Body
        if let Some(body) = self.body.clone() {
            response_bytes = [response_bytes, body].concat();
        }
        response_bytes
    }
}
