extern crate tokio;
extern crate tokio_tungstenite;

use log::info;
use std::{env};
use std::collections::HashMap;
use std::fmt::Error;
use std::net::{IpAddr, SocketAddr};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use tokio_tungstenite::tungstenite::{Message};

static DEFAULT_ADDR: &str = "127.0.0.1";
static DEFAULT_PORT: u16 = 9999;

type Tx = UnboundedSender<Message>;
type ClientMap = Arc<Mutex<HashMap<i32, Tx>>>;

#[tokio::main]
async fn main() -> Result<(), Error>{
    println!("hello world");

    let ip_str = env::args().nth(1).unwrap_or_else(|| String::from(DEFAULT_ADDR));
    let ip = IpAddr::V4(ip_str.parse().unwrap());

    let socket_addr = SocketAddr::new(ip, DEFAULT_PORT);

    let mut cnt = 0;
    let mut clientMap: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind(&socket_addr).await.expect("Not Bind to addr");
    while let res = listener.accept().await {
        match res {
            Ok((stream, _)) => {
                cnt += 1;
                tokio::spawn(handle_connection(stream, cnt, clientMap.clone()));
            },
            Err(_) => { panic!("stream match error"); }
        }
    };

    Ok(())
}

async fn handle_connection(stream: TcpStream, title: i32, mut clientMap: ClientMap) {
    let addr = stream.peer_addr().expect("connect failed");
    let websocket = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    // Log Info
    println!("WebSocket Client In : {} / {}", addr, title);

    // Insert peer
    let (tx, rx) = unbounded::<Message>();
    clientMap.lock().unwrap().insert(title, tx);

    let (writer, reader) = websocket.split();

    let broadcast_incoming  = reader.try_for_each(|msg| {
        println!("Recevie Message : {}", msg.to_text().unwrap());

        let clients = clientMap.lock().unwrap();
        let broadcast_iter = clients.iter().map(|(_, ws_sink)| ws_sink);

        let mut send_msg = String::from(title.to_string());
        send_msg.push_str(" : ");
        send_msg.push_str(msg.to_string().as_str());

        for recp  in broadcast_iter {
            recp.unbounded_send(Message::text(&send_msg)).unwrap();
        }

        future::ok(())
    });

    let receive_from_other = rx.map(Ok).forward(writer);
    pin_mut!(broadcast_incoming, receive_from_other);
    future::select(broadcast_incoming, receive_from_other).await;

    println!("{} disconnected", &addr);
    clientMap.lock().unwrap().remove(&title);
}
