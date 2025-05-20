use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = "ws://127.0.0.1:2000";

    let (ws_stream, _) = ClientBuilder::new().uri(url)?.connect().await?;
    let (mut sender, mut receiver) = ws_stream.split();

    let mut input = BufReader::new(tokio::io::stdin()).lines();

    println!("Connected to {url}");
    println!("Type messages and press Enter to send");

    loop {
        tokio::select! {
            line = input.next_line() => {
                match line? {
                    Some(text) => {
                        sender.send(Message::text(text)).await?;
                    }
                    None => break, // EOF
                }
            }
            msg = receiver.next() => {
                match msg {
                    Some(Ok(msg)) if msg.is_text() => {
                        if let Some(text) = msg.as_text() {
                            println!("Received: {text}");
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error: {e}");
                        break;
                    }
                    _ => break,
                }
            }
        }
    }

    println!("Disconnected from server.");
    Ok(())
}
