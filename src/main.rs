use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use std::path::PathBuf;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: rustwebserver PORT ROOT_FOLDER");
        return;
    }
    let port = &args[1];
    let root_folder = PathBuf::from(&args[2]);
    // Print startup information
    println!("Root folder: {}", folder.display());
    println!("Server listening on 0.0.0.0:{}", port);
    
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

 fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    println!("Request: {http_request:#?}");
}