use std::collections::HashMap;

use crate::{
    request::{HTTPMethod, Request},
    response::Response,
};

use regex::Regex;
#[derive(Clone, Default, Debug)]
pub struct Router {
    handlers: HashMap<String, StaticRoute>,
    dynamic_routes: Vec<DynamicRoute>,
}

#[derive(Clone, Debug)]
struct StaticRoute {
    handler: fn(Request, Response) -> Response,
    method: HTTPMethod,
}

#[derive(Clone, Debug)]
struct DynamicRoute {
    handler: fn(Request, Response) -> Response,
    regex: String,
    parts: Vec<DRoutePart>,
    method: HTTPMethod,
}
#[derive(Clone, Default, Debug)]
struct DRoutePart {
    index: usize,
    name: String,
}

#[derive(Debug)]

pub struct RouteDetails {
    pub handler: fn(Request, Response) -> Response,
    pub path_variables: HashMap<String, String>,
}

impl Router {
    pub fn new() -> Router {
        Router::default()
    }
    pub fn insert_route(
        &mut self,
        route: &str,
        handler: fn(Request, Response) -> Response,
        method: HTTPMethod,
    ) {
        let route = route.to_owned();
        if route.contains(':') {
            // Dynamic route
            self.insert_dynamic_route(route, handler, method)
        } else {
            self.handlers.insert(route, StaticRoute { handler, method });
        }
    }

    fn insert_dynamic_route(
        &mut self,
        route: String,
        handler: fn(Request, Response) -> Response,
        method: HTTPMethod,
    ) {
        // Tranform input into regex

        let mut droute = DynamicRoute {
            handler,
            parts: Vec::new(),
            regex: String::new(),
            method,
        };
        let mut regex = String::from(r"\/");
        let mut drouteparts = Vec::new();

        for (index, part) in route
            .clone()
            .split('/')
            .filter(|x| !x.is_empty())
            .enumerate()
        {
            if part.starts_with(':') {
                regex.push_str(r"[A-Z0-9a-z]+\/?");

                let droutepart = DRoutePart {
                    index,
                    name: part.split_once(':').unwrap().1.to_owned(),
                };
                drouteparts.push(droutepart)
            } else {
                regex.push_str(format!(r"({}\/?)", part).as_str());
            }
        }

        regex.push('$');
        droute.regex = regex;
        droute.parts = drouteparts;

        self.dynamic_routes.push(droute)
    }

    fn get_dynamic_route(&self, route: &str, method: &HTTPMethod) -> Option<RouteDetails> {
        for child in &self.dynamic_routes {
            let regex = Regex::new(&child.regex).unwrap();

            if &child.method != method {
                continue;
            }

            if regex.is_match(route) {
                // Get the individual parts now
                let mut hash = HashMap::new();

                let mut parts = child.parts.clone();
                parts.reverse();

                let mut next_index_opt = parts.pop();

                for (index, part) in route.split('/').filter(|x| !x.is_empty()).enumerate() {
                    if next_index_opt.is_none() {
                        break;
                    }
                    let next_index = next_index_opt.clone().unwrap();

                    if next_index.index == index {
                        hash.insert(next_index.name, part.to_owned());
                        next_index_opt = parts.pop();
                    }
                }

                let route_details = RouteDetails {
                    handler: child.handler,
                    path_variables: hash,
                };

                return Some(route_details);
            }
        }
        None
    }

    pub fn get_route(&self, route: &str, method: &HTTPMethod) -> Option<RouteDetails> {
        // Try to get a static route else dynamic else None
        if let Some(sroute) = self.handlers.get(route).map(|handler| handler.to_owned()) {
            if &sroute.method != method {
                return None;
            }

            let route_details = RouteDetails {
                handler: sroute.handler,
                path_variables: HashMap::new(),
            };
            return Some(route_details);
        }

        // Dynamic

        self.get_dynamic_route(route, method)
    }
}
