use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                loop {
                    let mut buffer = [0; 512];
                    match stream.read(&mut buffer) {
                        Ok(n) => {
                            if n == 0 {
                                break;
                            }
                            let request = String::from_utf8_lossy(&buffer[..n]);
                            println!("request: {}", request);
                            if request.contains("PING") {
                                let response = "+PONG\r\n";
                                stream.write(response.as_bytes()).unwrap();
                                stream.flush().unwrap();
                            }
                        }
                        Err(e) => {
                            println!("error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
