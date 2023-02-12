extern crate tokio;
extern crate tokio_tungstenite;

use log::info;
use std::{env};
use std::fmt::Error;
use std::net::{IpAddr, SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use futures_util::{future, StreamExt, TryStreamExt};
use tokio_tungstenite::tungstenite::WebSocket;

static DEFAULT_ADDR: &str = "127.0.0.1";
static DEFAULT_PORT: u16 = 9999;

#[tokio::main]
async fn main() -> Result<(), Error>{
    println!("hello world");

    let ip_str = env::args().nth(1).unwrap_or_else(|| String::from(DEFAULT_ADDR));
    let ip = IpAddr::V4(ip_str.parse().unwrap());

    let socket_addr = SocketAddr::new(ip, DEFAULT_PORT);

    let listener = TcpListener::bind(&socket_addr).await.expect("Not Bind to addr");
    while let res = listener.accept().await {
        match res {
            Ok((stream, _)) => {
                tokio::spawn(handle_connection(stream));
            },
            Err(_) => { panic!("stream match error"); }
        }
    };

    Ok(())
}

async fn handle_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("connect failed");
    let websocket = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    // Log Info
    info!("New Client In : {}", addr);

    let (writer, reader) = websocket.split();
    reader.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(writer)
        .await
        .expect("Failed to forward messages")
}
