use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
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
            maybe_msg = ws_stream.next() => {
                match maybe_msg {
                    Some(Ok(msg)) => {
                        if let Some(txt) = msg.as_text() {
                            println!("From client {addr} \"{txt}\"");
                            let _ = bcast_tx.send(format!("Ratih's Computer - From server: [{addr}]: {txt}"));
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error from {addr}: {e}");
                        break;
                    }
                    None => break,
                }
            }
    
            result = bcast_rx.recv() => {
                match result {
                    Ok(msg) => {
                        ws_stream.send(Message::text(msg)).await?;
                    }
                    Err(e) => {
                        eprintln!("Broadcast receive error for {addr}: {e}");
                        break;
                    }
                }
            }
        }
    }    

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on ws://127.0.0.1:8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from Ratih's computer: {addr:?}");

        let bcast_tx = bcast_tx.clone();

        tokio::spawn(async move {
            let (_req, ws_stream) = ServerBuilder::new().accept(socket).await.unwrap();
            if let Err(e) = handle_connection(addr, ws_stream, bcast_tx).await {
                eprintln!("Connection error with {addr:?}: {e}");
            }
        });
    }
}
