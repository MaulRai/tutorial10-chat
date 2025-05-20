use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) if msg.is_text() => {
                        if let Some(text) = msg.as_text() {
                            println!("Received from {addr}: {text}");
                            let _ = bcast_tx.send(text.to_string()); // ignore send error if no listener
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error from {addr}: {e}");
                        break;
                    }
                    _ => break,
                }
            }

            broadcast = bcast_rx.recv() => {
                match broadcast {
                    Ok(msg) => {
                        if ws_stream.send(Message::text(msg)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break, // if channel is closed
                }
            }
        }
    }

    println!("Connection closed: {addr}");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(32);
    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("Server listening on ws://127.0.0.1:2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client connected: {addr}");

        let bcast_tx = bcast_tx.clone();
        tokio::spawn(async move {
            match ServerBuilder::new().accept(socket).await {
                Ok((_req, ws_stream)) => {
                    if let Err(e) = handle_connection(addr, ws_stream, bcast_tx).await {
                        eprintln!("Error handling {addr}: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to upgrade connection from {addr}: {e}");
                }
            }
        });
    }
}