use crate::{request::Request, response::Response};
use itertools::Itertools;
use std::io::{Read, Write};

pub fn echo(request: Request, mut response: Response) -> Response {
    let (_, data) = request.target.rsplit_once('/').unwrap();

    response.set_header("Content-Type".to_string(), "text/plain".to_string());
    response.set_status(200, "OK".to_string());

    response.set_body(data.as_bytes().into());
    response
}

pub fn root(_request: Request, mut response: Response) -> Response {
    response.set_status(200, "OK".to_string());
    response
}

pub fn user_agent(request: Request, mut response: Response) -> Response {
    let user_agent = request.headers.get("User-Agent").unwrap();
    response.set_header("Content-Type".to_string(), "text/plain".to_string());
    response.set_status(200, "OK".to_string());
    response.set_body(user_agent.as_bytes().into());
    response
}

pub fn get_files(request: Request, mut response: Response) -> Response {
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
    println!("{}", path);
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
    response
}

pub fn post_files(request: Request, mut response: Response) -> Response {
    let (_, _file) = request.target.rsplit_once('/').unwrap();
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
    response
}
