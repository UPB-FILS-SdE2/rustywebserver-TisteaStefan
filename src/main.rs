use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::header::{HeaderMap, HeaderValue};
use std::convert::Infallible;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::process::Command;
use mime_guess;
use tokio::io::AsyncWriteExt; // Add this import for write_all
use std::process::Stdio; // Add this import for Stdio

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <port> <root_folder>", args[0]);
        std::process::exit(1);
    }

    let port: u16 = args[1].parse().expect("Invalid port number");
    let root_folder = Arc::new(args[2].clone());

    println!("Root folder: {}", root_folder);
    println!("Server listening on 0.0.0.0:{}", port);

    let make_svc = make_service_fn(move |_conn| {
        let root_folder = Arc::clone(&root_folder);
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let root_folder = Arc::clone(&root_folder);
                handle_request(req, root_folder)
            }))
        }
    });

    let addr = ([0, 0, 0, 0], port).into();
    let server = Server::bind(&addr).serve(make_svc);

    server.await?;
    Ok(())
}

async fn handle_request(
    req: Request<Body>,
    root_folder: Arc<String>,
) -> Result<Response<Body>, Infallible> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let full_path = format!("{}/{}", root_folder, path.trim_start_matches('/'));
    let headers = req.headers().clone();

    let response = match (method.as_str(), Path::new(&full_path).is_file(), path.as_str()) {
        ("GET", true, _) => handle_get_request(full_path, path).await,
        ("GET", false, _) if path.starts_with("/scripts/") => handle_script_request(full_path, path, "GET", headers).await,
        ("POST", false, _) if path.starts_with("/scripts/") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
            handle_script_request_with_body(full_path, path, "POST", headers, whole_body).await
        }
        _ => not_found_response(),
    };

    Ok(response)
}

async fn handle_get_request(full_path: String, path: String) -> Response<Body> {
    match fs::read(&full_path) {
        Ok(contents) => {
            let mime_type = mime_guess::from_path(&full_path).first_or_octet_stream().to_string();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime_type)
                .header("Connection", "close")
                .body(Body::from(contents))
                .unwrap();

            println!("GET 127.0.0.1 {} -> 200 (OK)", path);
            response
        }
        Err(_) => forbidden_response(),
    }
}

async fn handle_script_request(
    script_path: String,
    path: String,
    method: &str,
    headers: HeaderMap<HeaderValue>,
) -> Response<Body> {
    if Path::new(&script_path).is_file() {
        let mut cmd = Command::new(&script_path);
        set_env_vars(&mut cmd, headers, method, &path);

        let output = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output().await;
        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header("Connection", "close")
                        .body(Body::from(stdout.to_string()))
                        .unwrap();

                    println!("{} 127.0.0.1 {} -> 200 (OK)", method, path);
                    response
                } else {
                    internal_server_error_response()
                }
            }
            Err(_) => internal_server_error_response(),
        }
    } else {
        not_found_response()
    }
}

async fn handle_script_request_with_body(
    script_path: String,
    path: String,
    method: &str,
    headers: HeaderMap<HeaderValue>,
    body: hyper::body::Bytes,
) -> Response<Body> {
    if Path::new(&script_path).is_file() {
        let mut cmd = Command::new(&script_path);
        set_env_vars(&mut cmd, headers, method, &path);

        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn script");

        if let Some(mut stdin) = child.stdin.take() {
            tokio::spawn(async move {
                stdin.write_all(&body).await.expect("Failed to write to stdin");
            });
        }

        let output = child.wait_with_output().await;
        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header("Connection", "close")
                        .body(Body::from(stdout.to_string()))
                        .unwrap();

                    println!("{} 127.0.0.1 {} -> 200 (OK)", method, path);
                    response
                } else {
                    internal_server_error_response()
                }
            }
            Err(_) => internal_server_error_response(),
        }
    } else {
        not_found_response()
    }
}

fn set_env_vars(cmd: &mut Command, headers: HeaderMap<HeaderValue>, method: &str, path: &str) {
    for (key, value) in headers.iter() {
        if let Ok(val_str) = value.to_str() {
            cmd.env(key.as_str(), val_str);
        }
    }
    cmd.env("Method", method);
    cmd.env("Path", path);
}

fn forbidden_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("Connection", "close")
        .body(Body::from("<html>403 Forbidden</html>"))
        .unwrap()
}

fn not_found_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Connection", "close")
        .body(Body::from("<html>404 Not Found</html>"))
        .unwrap()
}

fn internal_server_error_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Connection", "close")
        .body(Body::from("<html>500 Internal Server Error</html>"))
        .unwrap()
}
