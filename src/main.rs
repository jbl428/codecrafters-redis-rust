use std::io::Write;
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                // send PONG
                let pong = "+PONG\r\n";
                _stream.write(pong.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
