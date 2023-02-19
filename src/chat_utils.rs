
pub mod server_manger {
    use std::collections::HashMap;
    use std::env;
    use std::net::{IpAddr, SocketAddr};
    use std::sync::{Arc, Mutex};
    use futures_channel::mpsc::{unbounded, UnboundedSender};
    use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio_tungstenite::tungstenite::Message;
    use uuid::Uuid;

    type Tx = UnboundedSender<Message>;
    type ClientMap = Arc<Mutex<HashMap<String, Tx>>>;

    static DEFAULT_ADDR: &str = "127.0.0.1";
    static DEFAULT_PORT: u16 = 9999;

    pub struct ChatServer {
        listener: TcpListener,
        clients: ClientMap,
    }

    impl ChatServer {
        pub async fn create() -> ChatServer {
            let ip_str = env::args().nth(1).unwrap_or_else(|| String::from(DEFAULT_ADDR));
            let ip = IpAddr::V4(ip_str.parse().unwrap());
            let socket_addr = SocketAddr::new(ip, DEFAULT_PORT);

            ChatServer {
                listener: TcpListener::bind(&socket_addr).await.expect("Not Bind to addr"),
                clients: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub async fn listening(&self) {
            while let res = self.listener.accept().await {
                match res {
                    Ok((stream, _)) => {
                        tokio::spawn(Self::handle_connection(stream, Uuid::new_v4().to_string(), self.clients.clone()));
                    },
                    Err(_) => { panic!("stream match error"); }
                }
            };
        }

        async fn handle_connection(stream: TcpStream, id_str: String, mut clients: ClientMap) {
            let id_clone = id_str.clone();
            let addr = stream.peer_addr().expect("connect failed");
            let websocket = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");

            // Log Info
            println!("WebSocket Client In : {} / {}", addr, id_str);

            // Insert peer
            let (tx, rx) = unbounded::<Message>();
            clients.lock().unwrap().insert(id_str, tx);

            let (writer, reader) = websocket.split();

            let broadcast_incoming  = reader.try_for_each(|msg| {
                println!("Recevie Message : {}", msg.to_text().unwrap());

                let clients = clients.lock().unwrap();
                let broadcast_iter = clients.iter().map(|(_, ws_sink)| ws_sink);

                let mut send_msg = String::from(&id_clone);
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
            clients.lock().unwrap().remove(&id_clone);
        }
    }
}
