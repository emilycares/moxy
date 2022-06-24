use std::convert::Infallible;
use std::{collections::HashMap, time::Duration};

use futures_util::StreamExt;
use reqwest::Url;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;

/// Calls the remote and returns WsMessages
pub async fn fetch_ws(
    url: &str,
    _header: HashMap<String, String>,
    client_messages: Vec<WsClientMessage>,
) -> Result<Vec<WsClientMessage>, Infallible> {
    let (mut socket, _response) =
        connect_async(Url::parse(url).unwrap())
            .await
            .expect("Can't connect");

    for m in client_messages {
        sleep(Duration::from_millis(m.offset)).await;
        match socket.get_mut().write(&m.content.into_boxed_slice()).await {
            Ok(data) => log::trace!("[WS] sent message to server: {:?}", data),
            Err(_) => log::trace!("[WS] Unable to send data to server"),
        }
    }

    let (_write, read) = socket.split();

    Ok(read
        .map(|message| {
            let data = message.unwrap().into_data();
            //tokio::io::stdout().write_all(&data).await.unwrap();
            WsClientMessage {
                offset: 0,
                content: data,
            }
        })
        .collect::<Vec<WsClientMessage>>()
        .await)
}

/// A Message that is sent or recevied on a websocket
pub struct WsClientMessage {
    /// When the message is sent
    pub offset: u64,
    /// Message content
    pub content: Vec<u8>,
}
