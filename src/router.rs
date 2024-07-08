use std::collections::HashMap;

use crate::{request::Request, response::Response};

pub struct Router {
    handlers: HashMap<String, fn(Request, Response) -> Response>,
}

impl Router {
    pub fn new() -> Router {
        Router {
            handlers: HashMap::new(),
        }
    }
    pub fn insert_route(&mut self, route: String, handler: fn(Request, Response) -> Response) {
        self.handlers.insert(route, handler);
    }

    pub fn get_route(&self, route: &str) -> Option<fn(Request, Response) -> Response> {
        self.handlers.get(route).map(|handler| handler.to_owned())
    }
}
