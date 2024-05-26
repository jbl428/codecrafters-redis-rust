use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::spawn;

use resp::tokenize;

use crate::command::{CommandContext, CommandDispatcher};
use crate::resp::RespToken;
use crate::store::Store;

mod command;
mod resp;
mod store;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let port = args
        .get(2)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(6379);
    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(address).await?;
    let store = Store::new();

    loop {
        let (mut stream, _) = listener.accept().await?;
        let store = store.clone();

        spawn(async move {
            let result = handle_connection(&mut stream, store).await;
            if let Err(e) = result {
                println!("error: {}", e);
            }
        });
    }
}

async fn handle_connection(
    stream: &mut tokio::net::TcpStream,
    store: Store,
) -> std::io::Result<()> {
    let mut buffer = [0; 512];

    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..n]);
        let response = handle_request(&request, store.clone());

        stream.write_all(response.to_string().as_bytes()).await?;
    }
}

fn handle_request(request: &str, store: Store) -> RespToken {
    match tokenize(request) {
        Ok((_, token)) => {
            let dispatcher = CommandDispatcher::new();
            dispatcher.dispatch(&CommandContext { token, store })
        }
        Err(_) => RespToken::SimpleError("parse error".to_string()),
    }
}
