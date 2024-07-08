use std::io::Write;

use flate2::{write::GzEncoder, Compression};

use crate::{
    request::{HTTPMethod, Request},
    response::Response,
    router::Router,
    services::{echo, get_files, post_files},
};

pub fn runner(request: Request, router: &Router) -> Response {
    println!("{:?}", request);

    let mut response = Response::new();

    // Direct matches
    if let Some(handler) = router.get_route(&request.target) {
        response = handler(request.clone(), response)
    } else {
        // For paths with dynamic values. better implementation is needed
        if request.target.starts_with("/echo/") {
            response = echo(request.clone(), response);
        }
        if request.target.starts_with("/files/") {
            // Get request
            match request.http_method {
                HTTPMethod::Get => response = get_files(request.clone(), response),
                HTTPMethod::Post => response = post_files(request.clone(), response),
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

    println!("{:?}", response);

    response
}
