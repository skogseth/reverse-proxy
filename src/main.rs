use std::net::SocketAddr;

use http::Uri;
use http_body_util::Empty;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    eprintln!("Listening on {addr}");
    loop {
        // Accept incoming connections (and log failing ones)
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(err) => {
                eprintln!("Failed to accept connection: {err}");
                continue;
            }
        };

        // Use an adapter to access something implementing `tokio::io`
        // traits as if they implement `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            // We bind the incoming connection to our service
            let server_connection = hyper::server::conn::http1::Builder::new();
            let future = server_connection.serve_connection(io, service_fn(handle_request));

            if let Err(err) = future.await {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    incoming_request: Request<Incoming>,
) -> Result<Response<Incoming>, anyhow::Error> {
    let address = SocketAddr::from(([127, 0, 0, 1], 8000));
    let uri_string = format!("http://{}{}", address, incoming_request.uri());
    let uri: Uri = uri_string.parse().unwrap();
    eprintln!("Redirecting to: {uri}");

    // Open a TCP connection to the remote host
    let stream = TcpStream::connect(address).await?;

    // Use an adapter to access something implementing `tokio::io`
    // traits as if they implement `hyper::rt` IO traits.
    let io = TokioIo::new(stream);

    // Create the Hyper client
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            eprintln!("Connection failed: {:?}", err);
        }
    });

    eprintln!("Handshake completed");

    // Create an HTTP request with an empty body and copy over headers
    let mut req = Request::get(incoming_request.uri());
    for (key, val) in incoming_request.headers() {
        req = req.header(key, val);
    }
    let request = req.body(Empty::<Bytes>::new())?;

    // Await the response...
    eprintln!("Forwarding request...");
    let res = sender.send_request(request).await?;
    eprintln!("Response status: {}", res.status());

    Ok(res)
}
