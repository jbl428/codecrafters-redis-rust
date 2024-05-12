use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::spawn;

use resp::tokenize;

use crate::command::CommandDispatcher;
use crate::resp::RespToken;

mod command;
mod resp;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;
        spawn(async move {
            let result = handle_connection(&mut stream).await;
            if let Err(e) = result {
                println!("error: {}", e);
            }
        });
    }
}

async fn handle_connection(stream: &mut tokio::net::TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 512];
    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..n]);
        let response = handle_request(&request);

        stream.write_all(response.to_string().as_bytes()).await?;
    }
}

fn handle_request(request: &str) -> RespToken {
    match tokenize(request) {
        Ok((_, token)) => {
            let dispatcher = CommandDispatcher::new();
            dispatcher.dispatch(&token)
        }
        Err(_) => RespToken::SimpleError("parse error".to_string()),
    }
}
