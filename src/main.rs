use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use http_server_starter_rust::{request::Request, router::Router, runner::runner, services};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let mut router = Router::new();

    router.insert_route("/".to_owned(), services::root);
    router.insert_route("/user-agent".to_owned(), services::user_agent);

    let router = Arc::new(router);
    for stream in listener.incoming() {
        let router = router.clone();
        std::thread::spawn(move || handle_stream(stream, router));
    }
}

fn handle_stream(stream: Result<TcpStream, std::io::Error>, router: Arc<Router>) {
    match stream {
        Ok(mut stream) => {
            let request = Request::new(&mut stream).unwrap();

            // Handle request

            let mut response = runner(request, &router);

            // Write response
            let response = response.as_bytes();

            stream.write_all(&response).unwrap();
            stream.flush().unwrap();
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
}
