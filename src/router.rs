//! Returns a respomse to a given request.
use futures_util::StreamExt;
use futures_util::{sink::SinkExt, stream::FuturesUnordered};
use hyper::upgrade::Upgraded;
use hyper_tungstenite::WebSocketStream;
use rayon::prelude::*;
use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use hyper_tungstenite::{tungstenite::Message, HyperWebsocket};
use tokio::sync::Mutex;

use crate::builder::request::util::header_map_to_hash_map;
use crate::configuration::Metadata;
use crate::{
    builder::{self, storage},
    configuration::{self, BuildMode, Configuration, RouteMethod, WsMessageType},
    data_loader,
};

/// Start webserver using hyper
pub async fn start() {
    let mut config = configuration::get_configuration().await;
    tracing::trace!("Config: {:?}", config);
    if config.host.is_none() {
        config.host = Configuration::default().host;
    }
    let addr: Result<SocketAddr, _> = config.host.as_ref().unwrap().parse();

    let config = Arc::new(Mutex::new(config));

    if let Ok(addr) = addr {
        let make_service = make_service_fn(move |_| {
            let config = config.clone();

            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let config = config.clone();
                    async move { check_ws(req, config).await }
                }))
            }
        });

        tracing::info!("Starting http server on http://{addr}");
        let server = Server::bind(&addr).serve(make_service);

        // Run this server for... forever!
        if let Err(e) = server.await {
            tracing::error!("server error: {}", e);
        }
    } else {
        tracing::error!("Unable to start application with an invalid host");
    }
}

/// Call data_loader or builder depending on if the route exists or not.
async fn endpoint(
    config_a: Arc<Mutex<Configuration>>,
    uri: &hyper::Uri,
    method: &hyper::Method,
) -> Result<Response<Body>, Infallible> {
    tracing::info!("{}", uri);
    let configc = config_a.clone();
    let mut config = configc.lock().await.to_owned();
    let (route, parameter) =
        configuration::get_route(&config.routes, uri, &RouteMethod::from(method));

    if let Some(route) = route {
        let data = data_loader::load(route, parameter);
        if let Some(data) = data.await {
            let response = Response::builder()
                .status(200)
                .header(
                    "content-type",
                    // unwrap is save here because there will never be data without a resource
                    storage::get_content_type(route.resource.as_ref().unwrap()),
                )
                .body(Body::from(data))
                .unwrap();

            Ok(response)
        } else {
            if let Some(x) = config.routes.iter().position(|c| c == route) {
                tracing::info!("Remove route because the file does not exist: {:?}", route);
                config.routes.remove(x);
            }

            if config.build_mode == Some(BuildMode::Write) {
                builder::core::build_response(config_a, uri, method).await
            } else {
                tracing::error!("Will build new route for missing file");
                let response = Response::builder().status(404).body(Body::empty()).unwrap();
                Ok(response)
            }
        }
    } else if config.build_mode == Some(BuildMode::Write) {
        builder::core::build_response(config_a, uri, method).await
    } else {
        tracing::info!("Resource not found and build mode disabled");
        let response = Response::builder().status(404).body(Body::empty()).unwrap();
        Ok(response)
    }
}

async fn check_ws(
    request: Request<Body>,
    config: Arc<Mutex<Configuration>>,
) -> Result<Response<Body>, Infallible> {
    let uri = request.uri().clone();
    let method = &request.method();
    if hyper_tungstenite::is_upgrade_request(&request) {
        if let Ok((response, websocket)) = hyper_tungstenite::upgrade(request, None) {
                let restponse_status = response.status().as_u16().clone();
                let response_headers = header_map_to_hash_map(response.headers()).clone();
            // Spawn a task to handle the websocket connection.
            tokio::spawn(async move {
                if let Err(e) = endpoint_ws(
                    &uri,
                    Some(Metadata {
                        code: restponse_status,
                        headers: response_headers,
                    }),
                    websocket,
                    config,
                )
                .await
                {
                    tracing::trace!("[WS] Error in websocket connection: {}", e);
                }
            });

            // Return the response so the spawned future can continue.
            Ok(response)
        } else {
            tracing::error!("Unable to upgrade connection");
            let response = Response::builder().status(404).body(Body::empty()).unwrap();
            Ok(response)
        }
    } else {
        endpoint(config, &uri, method).await
    }
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
async fn endpoint_ws(
    uri: &hyper::Uri,
    metadata: Option<Metadata>,
    websocket: HyperWebsocket,
    config_a: Arc<Mutex<Configuration>>,
) -> Result<(), Error> {
    let config = config_a.clone();
    let mut config = config.lock().await.to_owned();
    if let (Some(route), _parameter) =
        configuration::get_route(&config.routes, uri, &RouteMethod::WS)
    {
        let websocket = websocket.await?;

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        let startup_messages: Vec<Vec<u8>> = route
            .messages
            .par_iter()
            .filter(|c| c.kind == WsMessageType::Startup)
            .map(|c| data_loader::file_sync(c.location.as_str()))
            .filter(|c| c.is_ok())
            .map(|c| c.unwrap())
            .collect();

        for c in startup_messages {
            match std::str::from_utf8(&c) {
                Ok(m) => tx.send(Message::text(m)).await?,
                Err(_) => tx.send(Message::binary(c)).await?,
            };
        }
        let after_messages: Vec<(Duration, Vec<u8>)> = route
            .messages
            .par_iter()
            .filter(|c| c.kind == WsMessageType::After)
            .map(|c| (c.get_time(), data_loader::file_sync(c.location.as_str())))
            .filter(|c| c.0.is_some() && c.1.is_ok())
            .map(|c| (c.0.unwrap(), c.1.unwrap()))
            .collect();

        let mut tasks = FuturesUnordered::new();

        for m in after_messages {
            let tx = tx.clone();
            tasks.push(tokio::task::spawn(async move {
                tokio::time::sleep(m.0).await;

                let msg = m.1;

                match std::str::from_utf8(&msg) {
                    Ok(m) => tx.send(Message::text(m)).await.unwrap(),
                    Err(_) => tx.send(Message::binary(msg.clone())).await.unwrap(),
                }
            }));
        }

        tasks.push(tokio::task::spawn(async move {
            send_ws_messages(rx, websocket).await
        }));

        // execute all tasks
        while (tasks.next().await).is_some() {}
    } else if config.build_mode == Some(BuildMode::Write) {
        if let Some(remote) = &config.remote {
            tracing::trace!("Start ws build");
            let route = builder::ws::build_ws(uri, metadata, remote.to_owned(), websocket).await;
            if let Ok(route) = route {
                config.routes.push(route);
            }
            configuration::save_configuration(config.to_owned()).await?;
        } else {
            tracing::info!(
                "There is no configuration for the url: {}, and there is no remote specified",
                &uri.to_string()
            );
        }
    } else {
        tracing::info!(
            "There is no configuration for the url: {}, and the build_mode is not set to Write",
            &uri.to_string()
        );
    }

    Ok(())
}

async fn send_ws_messages(
    mut rx: tokio::sync::mpsc::Receiver<Message>,
    mut websocket: WebSocketStream<Upgraded>,
) {
    while let Some(message) = rx.recv().await {
        match websocket.send(message).await {
            Ok(_) => tracing::trace!("Sent message"),
            Err(_) => tracing::error!("Failed to send message"),
        }
    }
}
