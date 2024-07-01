use std::fs::File;
use std::io::{BufReader, Write};
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use rustls::server::{ServerConfig, ServerConnection};

fn main() {
    /*
    rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();
    */

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(&addr).unwrap();

    //let server_config = Arc::new(server_config());

    let mut server = TcpStream::connect("https://files.nordicsemi.com").unwrap();

    println!("Listening on {addr}");
    loop {
        let stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(err) => {
                eprintln!("failed to accept connection: {err}");
                continue;
            }
        };

        println!("Connection accepted!");
        //std::thread::spawn(move || handle_client(stream));
        /*
        let server_config = Arc::clone(&server_config);
        std::thread::spawn(move || {
            let mut conn = ServerConnection::new(server_config).unwrap();
            conn.complete_io(&mut stream).unwrap();

            // Check if the client provided a certificate
            match conn.peer_certificates() {
                Some(certs) if !certs.is_empty() => {
                    let mut stdout = std::io::stdout().lock();
                    writeln!(&mut stdout, "Client provided certificates:").unwrap();
                    for (i, cert) in certs.iter().enumerate() {
                        writeln!(&mut stdout, "Certificate {}: {:?}", i + 1, cert).unwrap();
                    }
                }
                _ => println!("Client did not provide a certificate."),
            }

            handle_client(stream);
        });
        */
    }
}

fn handle_client(mut client: TcpStream) {
    let mut server = TcpStream::connect("https://files.nordicsemi.com").unwrap();

    let mut client_to_server = client.try_clone().unwrap();
    let mut server_to_client = server.try_clone().unwrap();

    // Use threads to handle bi-directional communication
    std::thread::scope(|s| {
        s.spawn(move || {
            std::io::copy(&mut client_to_server, &mut server).unwrap();
        });

        s.spawn(move || {
            std::io::copy(&mut server_to_client, &mut client).unwrap();
        });
    });
}

fn server_config() -> ServerConfig {
    // Load certificates
    let certs = {
        let certfile = File::open("C:\\secrets\\server_cert.pem").unwrap();
        let mut reader = BufReader::new(certfile);
        rustls_pemfile::certs(&mut reader)
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
    };

    // Load private key
    let private_key = {
        let keyfile = File::open("C:\\secrets\\server_key.pem").unwrap();
        let mut reader = BufReader::new(keyfile);
        rustls_pemfile::private_key(&mut reader).unwrap().unwrap()
    };

    // Set up server configuration
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
        .unwrap()
}
