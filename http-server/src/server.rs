use std::{io::Write, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use crate::{request::Request, response::Response, router::Router};
use flate2::{write::GzEncoder, Compression};

pub struct Server {
    router: Router,
}

impl Server {
    pub fn new(router: Router) -> Self {
        Server { router }
    }

    pub async fn listen(&self, port: u16) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .unwrap();

        let router = Arc::new(RwLock::new(self.router.clone()));
        while let Ok((stream, _addr)) = listener.accept().await {
            let router = router.clone();
            tokio::spawn(handle_stream(stream, router));
        }
    }
}

async fn handle_stream(mut stream: TcpStream, router: Arc<RwLock<Router>>) {
    let router = router.read().await;

    let mut request = Request::new(&mut stream).await.unwrap();

    let router_details = router.get_route(&request.target, &request.http_method);
    let mut response = Response::new();
    println!("{:?}", request);

    if router_details.is_none() {
        stream.write_all(&response.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
        return;
    }

    let router_details = router_details.unwrap();

    request.path_variables = router_details.path_variables;

    let handler = router_details.handler;

    let mut response = handler(request.clone(), response);

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

    stream.write_all(&response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}
