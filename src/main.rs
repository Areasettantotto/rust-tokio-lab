#![deny(warnings)]

use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::Full;
use hyper::header::CONTENT_TYPE;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use tokio::net::TcpListener;

mod support;
use support::TokioIo;

// Minimal hello handler adapted from hyper examples
async fn hello(_: Request<impl hyper::body::Body>) -> Result<Response<Full<Bytes>>, Infallible> {
    let body = Full::new(Bytes::from(
        "Ciao dal server Rust con Tokio + Hyper 1.x! 🚀",
    ));
    let resp = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(body)
        .unwrap();
    Ok(resp)
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (tcp, _) = listener.accept().await?;

        // Spawn a new task to handle the connection
        tokio::task::spawn(async move {
            // Convert tokio TcpStream to a compat type that implements the traits
            // hyper expects (Read + Write). Requires tokio-util feature `compat`.
            let io = TokioIo::new(tcp);

            // Serve the connection with HTTP1 and our `hello` service
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(hello))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
