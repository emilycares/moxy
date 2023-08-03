use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use futures_util::{
    lock::Mutex,
    pin_mut,
    stream::{FuturesUnordered, SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use hyper::{http::HeaderName, upgrade::Upgraded, Request};
use hyper_tungstenite::{tungstenite::Message, WebSocketStream};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async_tls_with_config, tungstenite::protocol::WebSocketConfig, MaybeTlsStream,
};

use crate::{
    builder::request,
    configuration::{Metadata, Route, RouteMethod, WsMessagType},
};

use super::storage;

/// A Message that is sent or received on a websocket
#[derive(Debug, Clone)]
pub struct WsClientMessage {
    /// When the message is sent
    pub offset: u64,
    /// Message content
    pub content: Vec<u8>,
    /// Message type
    pub message_type: WsMessagType,
}

impl WsClientMessage {
    /// Add from Message
    pub fn from(message: Message, offset: u64) -> Self {
        match message {
            Message::Text(m) => Self {
                offset,
                content: m.as_str().as_bytes().to_vec(),
                message_type: WsMessagType::Text,
            },
            Message::Binary(m) => Self {
                offset,
                content: m,
                message_type: WsMessagType::Binary,
            },
            Message::Ping(m) => Self {
                offset,
                content: m,
                message_type: WsMessagType::Ping,
            },
            Message::Pong(m) => Self {
                offset,
                content: m,
                message_type: WsMessagType::Pong,
            },
            Message::Close(_) => Self {
                offset,
                content: vec![],
                message_type: WsMessagType::Close,
            },
            Message::Frame(_) => Self {
                offset,
                content: vec![],
                message_type: WsMessagType::Frame,
            },
        }
    }
}


/// generate route for a websocket
pub async fn build_ws(
    uri: &str,
    metadata: Option<Metadata>,
    remote: impl Into<String> + std::marker::Send + 'static,
    websocket: hyper_tungstenite::HyperWebsocket,
    no_ssl_check: bool,
) -> Result<Route, u8> {
    let path = uri;
    let mut route = Route {
        method: RouteMethod::WS,
        metadata: metadata.clone(),
        path: path.to_string(),
        resource: None,
        messages: vec![],
    };

    let empty_remote_messages: Vec<WsClientMessage> = vec![];
    let remote_messages = Arc::new(Mutex::new(empty_remote_messages));

    let websocket = match websocket.await {
        Ok(w) => w,
        Err(_) => todo!(),
    };
    let dur = Duration::from_secs(10);

    // Contains all user messages
    let (tx_u, rx_u) = tokio::sync::mpsc::channel(32);
    // Contains all remote messages
    let (tx_r, rx_r) = tokio::sync::broadcast::channel(32);
    let mut rx_r2 = tx_r.subscribe();

    let mut tasks = FuturesUnordered::new();

    // collect user input
    tasks.push(tokio::task::spawn(async move {
        let (write, read) = websocket.split();
        tracing::trace!("connect user");

        tokio::select! {
            _ = read_ws_client(read, tx_u) => {},
            _ = send_ws_client(write, rx_r) => {}
        }
    }));

    let uri = uri.to_string();

    // connect to remote
    tasks.push(tokio::task::spawn(async move {
        tracing::trace!("connect to remote");
        let url = get_ws_url(&request::util::get_url_str(&uri, &remote.into()));
        tracing::trace!("{:?}", url);
        let request = get_request(url, metadata);

        match connect_async_tls_with_config(
            request,
            Some(WebSocketConfig::default()),
            true,
            get_tls_connector(no_ssl_check),
        )
        .await
        {
            //match connect_async(url).await {
            Ok((socket, _response)) => {
                let (write, read) = socket.split();

                tokio::select! {
                    _ = read_ws_remote(read, tx_r) => {
                        tracing::trace!("Build done read");
                    }
                    _ = send_ws_remote(write, rx_u) => {
                        tracing::trace!("Build done send");
                    }
                }
            }
            Err(e) => {
                tracing::error!("Unable to connect to the websocket: {:#?}", e);
            }
        }
    }));
    let remote_messages2 = remote_messages.clone();
    // save messages
    tasks.push(tokio::task::spawn(async move {
        tracing::trace!("Save messages");
        let start = Instant::now();
        let remote_messages2 = remote_messages2.clone();
        loop {
            match rx_r2.recv().await {
                Ok(message) => {
                    let offset = start.elapsed().as_secs();
                    let mut messages = remote_messages2.lock().await;
                    messages.push(WsClientMessage::from(message.clone(), offset));
                }
                Err(_) => (),
            }
        }
    }));

    let pull = tasks.next();
    pin_mut!(pull);

    if tokio::time::timeout(dur, &mut pull).await.is_err() {
        println!("Taking more than ");
        tasks.iter().for_each(|t| t.abort());
    }

    tracing::trace!("Tasks done");

    //// execute all tasks
    //while (tasks.next().await).is_some() {
    //if start.elapsed() > dur {
    //tracing::trace!("[WS] Done");
    //break;
    //}
    //}

    let messages = remote_messages.lock().await.to_owned();
    route.messages = storage::save_ws_client_message(&path, messages).await;

    Ok(route)
}

/// Send rx to remote
pub async fn send_ws_remote(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut rx: tokio::sync::mpsc::Receiver<Message>,
) {
    loop {
        match rx.recv().await {
            Some(message) => match write.send(message).await {
                Ok(_data) => tracing::trace!("[WS] sent message to server"),
                Err(_) => tracing::trace!("[WS] Unable to send data to server"),
            },
            None => (),
        }
    }
}

/// Read from remote
pub async fn read_ws_remote(
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    tx: tokio::sync::broadcast::Sender<Message>,
) {
    loop {
        match read.next().await {
            Some(Ok(message)) => {
                if let Ok(_) = tx.send(message) {
                    tracing::trace!("Sent message to remote");
                } else {
                    tracing::error!("Unable to send message to remote");
                }
            }
            Some(Err(err)) => {
                tracing::error!("Got error while reading remote websocket: {err:?}");
            }
            None => (),
        }
    }
}

/// write user input to tx
pub async fn read_ws_client(
    mut read: SplitStream<WebSocketStream<Upgraded>>,
    tx: tokio::sync::mpsc::Sender<Message>,
) {
    loop {
        match read.next().await {
            Some(Ok(message)) => {
                if let Ok(_) = tx.send(message).await {
                    tracing::trace!("got user input");
                } else {
                    tracing::error!("Unable to process user input");
                }
            }
            Some(Err(err)) => {
                tracing::error!("Got error while reading client websocket: {err:?}");
            }
            None => (),
        }
    }
}

/// write rx to user input
pub async fn send_ws_client(
    mut write: SplitSink<WebSocketStream<Upgraded>, Message>,
    mut rx: tokio::sync::broadcast::Receiver<Message>,
) {
    loop {
        match rx.recv().await {
            Ok(message) => {
                if let Ok(_data) = write.send(message).await {
                    tracing::trace!("[WS] sent message to client");
                } else {
                    tracing::trace!("[WS] Unable to send data to client");
                }
            }
            Err(err) => {
                tracing::error!("Got error while messaging to websocket: {err}");
            }
        }
    }
}

/// Change protocol to ws
pub fn get_ws_url(url: &str) -> String {
    let mut url = String::from(url);
    if url.starts_with("http") {
        url.replace_range(..4, "ws");
    }

    url
}

fn get_tls_connector(no_ssl_check: bool) -> Option<tokio_tungstenite::Connector> {
    let Ok(connector) = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(no_ssl_check)
        .danger_accept_invalid_hostnames(no_ssl_check)
        .build() else {
        tracing::error!("Unable to create tls connector");
        return None;
    };

    Some(tokio_tungstenite::Connector::NativeTls(connector))
}

/// Creates a reuest with the specifed url and header
fn get_request(url: String, metadata: Option<Metadata>) -> Request<()> {
    let mut request = Request::builder().method("GET").uri(url);

    if let Some(metadata) = metadata {
        if let Some(request_header) = request.headers_mut() {
            for (key, value) in metadata.header {
                if let Some(key) = key {
                    request_header.insert::<HeaderName>(key.into(), value);
                }
            }
        }
    }

    request.body(()).unwrap()
}
