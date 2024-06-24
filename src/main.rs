use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io::prelude::*;

#[tokio::main]
async fn main()  {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <port> <root_folder>", args[0]);
        std::process::exit(1);
    }

    // Extract port number and root folder from command-line arguments
    let port = args[1].parse::<u16>().expect("Invalid port number");
    let path = args[2].clone();
    // Print startup information
    println!("Root folder: {}", path);
    println!("Server listening on 0.0.0.0:{}", port);

    // Start TCP listener
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port));
    
    for stream in listener.unwrap().incoming() {
        let stream = stream.unwrap();
        handle_connection(stream , path.clone() );
    }

}


fn handle_connection(mut stream: TcpStream, path: String) -> io::Result<()>{
    let mut buffer=[1; 1024];
    stream.read(&mut buffer).unwrap();
    let req = String::from_utf8_lossy(&buffer[..]).to_string();
    let mut lines = req.split("\r\n");
    let req_line = lines.next().ok_or("err");
    let mut req_parts = req_line.unwrap().split_whitespace();
    let req_type = req_parts.next().ok_or("Missing reqtype").unwrap().to_string();
    let req_path = req_parts.next().ok_or("Missing path").unwrap().to_string();
    let mut headers = Vec::new();
   // Skip the HTTP version
    req_parts.next().ok_or("Missing HTTP version").unwrap();
    for line in lines.clone() {
        if line.is_empty() {
             break;
            }
         headers.push(line.to_string());
    }
    let body = lines.clone().collect::<Vec<&str>>().join("\r\n");
    let body = if body.is_empty() { None } else { Some(body) }.unwrap();
    
    
    
    let mut full_path=path.clone();
   // full_path.push_str(req_path.as_str());
    let full_P=Path::new(&full_path);
    let extension=match full_P.extension().and_then(|ext| ext.to_str()) {
        Some("txt") => "text/plain; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("jpg") | Some("jpeg")  => "image/jpeg",
        Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("zip") => "application/zip",
        _ => "application/octet-stream",
    };
    if req_path.starts_with("/..") || req_path.starts_with("/forbidden") {
        println!("GET 127.0.0.1 {} -> 403 (Forbidden)", path);
        let response = b"HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n<html>403 Forbidden</html>";
        stream.write_all(response)?;
    }else{
        println!("{}",full_path);
    match fs::read(&full_path){
        Ok(content)=>{
            println!("GET 127.0.0.1 {} -> 200 (OK)", req_path);
            println!("GET 127.0.0.1 {} -> 200 (OK)", req_path.clone());
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",extension);
            let response_to = response.as_bytes();
            stream.write_all(response_to).unwrap();
            stream.write_all(&content).unwrap();
            stream.flush().unwrap();
        }
        Err(_e) => {
            println!("why");
            println!("GET 127.0.0.1 {} -> 404 (Not Found)", full_path);
            let response = b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n<html>404 Not Found</html>";
            stream.write_all(response).unwrap();
            stream.flush().unwrap();
        }
    }
    
    }


    

    Ok(())
}