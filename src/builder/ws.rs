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
use hyper::upgrade::Upgraded;
use hyper_tungstenite::{tungstenite::Message, WebSocketStream};
use reqwest::Url;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream};

use crate::{
    builder::request,
    configuration::{Route, RouteMethod},
};

use super::storage;

/// generate route for a websocket
pub async fn build_ws(
    uri: &hyper::Uri,
    remote: String,
    websocket: hyper_tungstenite::HyperWebsocket,
) -> Result<Route, u8> {
    let path = uri.path().to_string();
    let mut route = Route {
        method: RouteMethod::WS,
        path: path.clone(),
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
        log::trace!("connect user");

        tokio::select! {
            _ = read_ws_client(read, tx_u) => {},
            _ = send_ws_client(write, rx_r) => {}
        }
    }));

    let uri = uri.to_string();

    // connect to remote
    tasks.push(tokio::task::spawn(async move {
        log::trace!("connect to remote");
        let url = get_ws_url(&request::util::get_url_str(&uri, &remote));
        log::trace!("{}", url);
        if let Ok(url) = Url::parse(&url) {
            log::trace!("{:?}", url);
            match connect_async(url).await {
                Ok((socket, _response)) => {
                    let (write, read) = socket.split();

                    tokio::select! {
                        _ = read_ws_remote(read, tx_r) => {
                            log::trace!("Build done read");
                        }
                        _ = send_ws_remote(write, rx_u) => {
                            log::trace!("Build done send");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Unable to connect to the websocket: {:#?}", e);
                }
            }
        } else {
            //log::error!("Unable to connect to the websocket: {}", url);
        }
    }));
    let remote_messages2 = remote_messages.clone();
    // save messages
    tasks.push(tokio::task::spawn(async move {
        log::trace!("Save messages");
        let start = Instant::now();
        let remote_messages2 = remote_messages2.clone();
        while let Ok(message) = rx_r2.recv().await {
            let offset = start.elapsed().as_secs();
            let mut messages = remote_messages2.lock().await;
            messages.push(WsClientMessage::from(message.clone(), offset));
            //tx_u.send(message);
        }
    }));

    let pull = tasks.next();
    pin_mut!(pull);

    if tokio::time::timeout(dur, &mut pull).await.is_err() {
        println!("Taking more than ");
        tasks.iter().for_each(|t| t.abort())
    }

    log::trace!("Tasks done");

    //// execute all tasks
    //while (tasks.next().await).is_some() {
    //if start.elapsed() > dur {
    //log::trace!("[WS] Done");
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
    while let Some(message) = rx.recv().await {
        match write.send(message).await {
            Ok(_data) => log::trace!("[WS] sent message to server"),
            Err(_) => log::trace!("[WS] Unable to send data to server"),
        }
    }
    log::trace!("Sent all messages");
}

/// Read from remote
pub async fn read_ws_remote(
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    tx: tokio::sync::broadcast::Sender<Message>,
) {
    while let Some(Ok(message)) = read.next().await {
        match tx.send(message) {
            Ok(_) => log::trace!("Sent message to remote"),
            Err(_) => log::error!("Unable to send message to remote"),
        }
    }
}

/// write user input to tx
pub async fn read_ws_client(
    mut read: SplitStream<WebSocketStream<Upgraded>>,
    tx: tokio::sync::mpsc::Sender<Message>,
) {
    while let Some(Ok(message)) = read.next().await {
        match tx.send(message).await {
            Ok(_) => log::trace!("got user input"),
            Err(_) => log::error!("Unable to process user input"),
        }
    }
}

/// write rx to user input
pub async fn send_ws_client(
    mut write: SplitSink<WebSocketStream<Upgraded>, Message>,
    mut rx: tokio::sync::broadcast::Receiver<Message>,
) {
    while let Ok(message) = rx.recv().await {
        match write.send(message).await {
            Ok(_data) => log::trace!("[WS] sent message to client"),
            Err(_) => log::trace!("[WS] Unable to send data to client"),
        }
    }
    log::trace!("Sent all messages");
}

/// Change protocol to ws
pub fn get_ws_url(url: &str) -> String {
    let mut url = String::from(url);
    if url.starts_with("http") {
        url.replace_range(..4, "ws");
    }

    url
}

/// A Message that is sent or recevied on a websocket
#[derive(Debug, Clone)]
pub struct WsClientMessage {
    /// When the message is sent
    pub offset: u64,
    /// Message content
    pub content: Vec<u8>,
    /// If data is not a string
    pub binary: bool,
}

impl WsClientMessage {
    /// Add from Message
    pub fn from(message: Message, offset: u64) -> Self {
        match message {
            Message::Text(m) => Self {
                offset,
                content: m.as_str().as_bytes().to_vec(),
                binary: false,
            },
            Message::Binary(m) => Self {
                offset,
                content: m,
                binary: true,
            },
            _ => Self {
                offset,
                content: vec![],
                binary: true,
            },
        }
    }
}

impl From<Message> for WsClientMessage {
    fn from(_: Message) -> Self {
        todo!()
    }
}
