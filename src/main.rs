extern crate tokio;
extern crate tokio_tungstenite;

pub mod chat_utils;

use std::fmt::Error;
use futures_util::FutureExt;
use chat_utils::server_manger::ChatServer;

#[tokio::main]
async fn main() -> Result<(), Error>{
    let server = ChatServer::create().await;
    server.listening().await;

    Ok(())
}

