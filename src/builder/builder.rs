use std::{
    collections::HashMap,
    convert::Infallible,
    sync::Arc,
    time::{Duration, Instant},
};

use futures_util::StreamExt;
use hyper::{Body, Response};
use hyper_tungstenite::tungstenite::Message;
use tokio::sync::Mutex;

use crate::configuration::{BuildMode, Configuration, Route, RouteMethod};

use super::{
    request::{self, ws::WsClientMessage},
    storage,
};

/// The data structure that will contain all relevant data. To easily convert a request to a response
/// without doing a huge workaround.
pub struct ResourceData {
    /// HTTP method
    pub method: RouteMethod,
    /// HTTP heades
    pub headers: HashMap<String, String>,
    /// HTTP status code
    pub code: u16,
    /// HTTP body
    pub payload: Option<Vec<u8>>,
}

/// Handles unknown routes. It accomplishes that with creating HTTP request and saving the response
/// into a file. It also modifies the configuration in order to not call this function with the
/// same URL again.
pub async fn build_response(
    config_a: Arc<Mutex<Configuration>>,
    uri: &hyper::Uri,
    method: &hyper::Method,
) -> Result<Response<Body>, Infallible> {
    let config_b = config_a.clone();
    let config = config_b.lock().await.to_owned();
    if let Some(build_mode) = &config.build_mode {
        if let Some(remote) = &config.remote {
            let response = request::http::fetch_http(
                RouteMethod::from(method),
                request::util::get_url(uri, remote),
                None,
                HashMap::new(),
            )
            .await;

            match response {
                Some(response) => {
                    if let Some(body) = response.payload {
                        if response.code != 404 && build_mode == &BuildMode::Write {
                            storage::save(
                                &response.method,
                                uri.path(),
                                body.clone(),
                                &response.headers,
                                config_a,
                            )
                            .await
                            .unwrap()
                        }

                        get_response(response.headers, response.code, Body::from(body))
                    } else {
                        get_response(response.headers, response.code, Body::empty())
                    }
                }
                None => {
                    log::error!("No response from endpoint");
                    let response = Response::builder().status(404).body(Body::empty()).unwrap();
                    Ok(response)
                }
            }
        } else {
            log::error!("Resource not found and no remove specified");
            let response = Response::builder().status(404).body(Body::empty()).unwrap();
            Ok(response)
        }
    } else {
        log::info!("Resource not found and build mode disabled");
        let response = Response::builder().status(404).body(Body::empty()).unwrap();
        Ok(response)
    }
}

/// Returns a respinse with headers and a code
pub fn get_response(
    headers: HashMap<String, String>,
    code: u16,
    body: Body,
) -> Result<Response<Body>, Infallible> {
    let mut response = Response::builder().status(code);

    for (key, value) in headers.into_iter() {
        response = response.header(key, value);
    }

    Ok(response.body(body).unwrap())
}

/// generate route for a websocket
pub async fn build_ws(
    uri: &hyper::Uri,
    remote: String,
    websocket: hyper_tungstenite::HyperWebsocket,
) -> Route {
    let path = uri.path().to_string();
    let mut route = Route {
        method: RouteMethod::WS,
        path: path.clone(),
        resource: None,
        messages: vec![],
    };

    let mut client_messages: Vec<WsClientMessage> = vec![];

    let mut websocket = match websocket.await {
        Ok(w) => w,
        Err(_) => todo!(),
    };
    // record messages from client
    let start = Instant::now();
    let dur = Duration::from_secs(5);
    while start.elapsed() < dur {
        log::trace!("{:#?} < {:#?} = {:#?}", start.elapsed(), dur, start.elapsed() < dur);
        if let Some(Ok(message)) = websocket.next().await {
            let differece = start.elapsed();
            let offset = differece.as_secs();
            match message {
                Message::Text(msg) => {
                    log::trace!("[WS] Received text message: {}", msg);
                    let message = WsClientMessage {
                        offset,
                        content: msg.as_bytes().to_vec(),
                        binary: false,
                    };
                    client_messages.push(message);
                }
                Message::Binary(msg) => {
                    log::trace!("[WS] Received binary message: {:02X?}", msg);
                    let message = WsClientMessage {
                        offset,
                        content: msg,
                        binary: true,
                    };
                    client_messages.push(message);
                }
                Message::Ping(msg) => {
                    // No need to send a reply: tungstenite takes care of this for you.
                    log::trace!("[WS] Received ping message: {:02X?}", msg);
                }
                Message::Pong(msg) => {
                    log::trace!("[WS] Received pong message: {:02X?}", msg);
                }
                Message::Close(msg) => {
                    // No need to send a reply: tungstenite takes care of this for you.
                    if let Some(msg) = &msg {
                        println!(
                            "Received close message with code {} and message: {}",
                            msg.code, msg.reason
                        );
                    } else {
                        log::trace!("[WS] Received close message");
                    }

                    break;
                }
                Message::Frame(msg) => {
                    log::trace!("[WS] Received pong message: {:02X?}", msg);
                }
            }
        }
    }

    log::info!("The messages: {:?}", client_messages);

    if websocket.close(None).await.is_err() {
        log::error!("Unable to close websocket");
    }

    let messages = request::ws::fetch_ws(
        &request::util::get_url(uri, &remote),
        HashMap::new(),
        client_messages,
    )
    .await;

    log::trace!("{:#?}", messages);

    if let Ok(messages) = messages {
        log::trace!("messages to save: {:#?}", messages);
        route.messages = storage::save_ws_client_message(&path, messages).await;
    }

    route
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{builder::request, configuration::RouteMethod};

    #[tokio::test]
    async fn requrest_no_body() {
        let _reponse = request::http::fetch_http(
            RouteMethod::GET,
            "http://example.com".to_string(),
            None,
            HashMap::new(),
        )
        .await
        .unwrap();
    }
}
