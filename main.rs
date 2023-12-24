use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    // Configuration
    let host = "127.0.0.1";
    let port = "22";
    let listen = "8880";

    // Listen new bridged port
    let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", listen))?;
    println!("Server started on port: {}", listen);
    println!("Redirecting requests to: {} at port {}", host, port);

    // looping
    for stream in listener.incoming() {
        match stream {
            Ok(client) => {
                // Log Output
                println!("Connection received from {}", client.peer_addr()?);

                // Spawn a new thread to handle the connection
                thread::spawn(move || {
                    handle_connection(client, host, port);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_connection(mut client: TcpStream, host: &str, port: &str) {
    // Dial Target in tcp with time out
    if let Ok(mut target) = TcpStream::connect_timeout(&format!("{}:{}", host, port), Duration::from_secs(5)) {
        // Return HTTP Response Switching Protocols to client
        let response = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n";
        if let Err(e) = client.write_all(response.as_bytes()) {
            eprintln!("Error writing response to client: {}", e);
            return;
        }

        // Copy client request to target (Forward)
        let forward_handle = thread::spawn(move || {
            if let Err(e) = io::copy(&mut client, &mut target) {
                eprintln!("Error copying data from client to destination server: {}", e);
            }
        });

        // Copy Return from target to client
        if let Err(e) = io::copy(&mut target, &mut client) {
            eprintln!("Error copying data from destination server to client: {}", e);
        }

        // Wait for the forward thread to finish
        if let Err(e) = forward_handle.join() {
            eprintln!("Error joining forward thread: {}", e);
        }

        // Log output
        println!("Connection terminated for {}", client.peer_addr().unwrap());
    } else {
        eprintln!("Error connecting to target server");
    }
}
