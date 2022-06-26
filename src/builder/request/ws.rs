use std::time::Instant;
use std::{collections::HashMap, time::Duration};

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use hyper_tungstenite::tungstenite::Message;
use hyper_tungstenite::WebSocketStream;
use reqwest::Url;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, MaybeTlsStream};

/// Calls the remote and returns WsMessages
pub async fn fetch_ws(
    url: &str,
    _header: HashMap<String, String>,
    client_messages: Vec<WsClientMessage>,
) -> Result<Vec<WsClientMessage>, u8> {
    let url = get_ws_url(url);
    log::trace!("{}", url);
    if let Ok(url) = Url::parse(url.as_str()) {
        match connect_async(url).await {
            Ok((socket, _response)) => {
                let (write, read) = socket.split();

                let mut messages: Vec<WsClientMessage> = vec![];

                tokio::select! {
                    a = read_ws_messages(read) => {
                        log::trace!("Got all messages");
                        messages = a;
                    }
                    _ = send_ws_messages(write, client_messages) => {
                        log::trace!("Sent all messages");
                    }
                }

                Ok(messages)
            }
            Err(e) => {
                log::error!("Unable to connect to the websocket: {:#?}", e);
                Err(2)
            }
        }
    } else {
        log::error!("Unable to connect to the websocket: {}", url);
        Err(1)
    }
}

async fn send_ws_messages(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    messages: Vec<WsClientMessage>,
) {
    for m in messages {
        sleep(Duration::from_secs(m.offset)).await;
        match write.send(Message::Binary(m.content)).await {
            Ok(_data) => log::trace!("[WS] sent message to server"),
            Err(_) => log::trace!("[WS] Unable to send data to server"),
        }
    }
    sleep(Duration::from_secs(500)).await;
}

async fn read_ws_messages(
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Vec<WsClientMessage> {
    let mut messages = vec![];
    let start = Instant::now();
    let dur = Duration::from_secs(5);
    while start.elapsed() < dur {
        if let Some(Ok(message)) = read.next().await {
            let differece = start.elapsed();
            let data = message.into_data();
            messages.push(WsClientMessage {
                offset: differece.as_secs(),
                content: data,
            });
        }
    }

    messages
}

fn get_ws_url(url: &str) -> String {
    let mut url = String::from(url);
    if url.starts_with("http") {
        url.replace_range(..4, "ws");
    }

    return url;
}

/// A Message that is sent or recevied on a websocket
#[derive(Debug)]
pub struct WsClientMessage {
    /// When the message is sent
    pub offset: u64,
    /// Message content
    pub content: Vec<u8>,
}
