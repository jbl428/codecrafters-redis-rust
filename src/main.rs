mod resp;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                spawn(move || { handle_connection(&mut stream).unwrap(); });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) -> std::io::Result<()> {
    loop {
        let mut buffer = [0; 512];
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    return Ok(());
                }
                let request = String::from_utf8_lossy(&buffer[..n]);
                if request.contains("PING") {
                    let response = "+PONG\r\n";
                    stream.write_all(response.as_bytes())?;
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
