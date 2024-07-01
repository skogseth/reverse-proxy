use std::net::{SocketAddr, TcpListener, TcpStream};

fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(&addr).unwrap();

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
        std::thread::spawn(move || handle_client(stream));
    }
}

fn handle_client(mut client: TcpStream) {
    let mut server = TcpStream::connect("127.0.0.1:8000").unwrap();

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
