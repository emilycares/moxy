//! Returns a respomse to a given request.
use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use rayon::prelude::*;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use hyper_tungstenite::{tungstenite::Message, HyperWebsocket};
use tokio::sync::Mutex;

use crate::{
    builder::{self, storage},
    configuration::{self, BuildMode, Configuration, RouteMethod, WsMessageType},
    data_loader,
};

/// Start webserver using hyper
pub async fn start() {
    let mut config = configuration::get_configuration().await;
    log::trace!("Config: {:?}", config);
    if config.host == None {
        config.host = Some("127.0.0.1:8080".to_string());
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

        log::info!("Starting http server");
        let server = Server::bind(&addr).serve(make_service);

        // Run this server for... forever!
        if let Err(e) = server.await {
            log::error!("server error: {}", e);
        }
    } else {
        log::error!("Unable to start application with an invalid host");
    }
}

/// Call data_loader or builder depending on if the route exists or not.
async fn endpoint(
    config_a: Arc<Mutex<Configuration>>,
    uri: &hyper::Uri,
    method: &hyper::Method,
) -> Result<Response<Body>, Infallible> {
    log::info!("{}", uri);
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
                log::info!("Remove route because the file does not exist: {:?}", route);
                config.routes.remove(x);
            }

            if config.build_mode == Some(BuildMode::Write) {
                builder::builder::build_response(config_a, uri, method).await
            } else {
                log::error!("Will build new route for missing file");
                let response = Response::builder().status(404).body(Body::empty()).unwrap();
                Ok(response)
            }
        }
    } else if config.build_mode == Some(BuildMode::Write) {
        builder::builder::build_response(config_a, uri, method).await
    } else {
        log::info!("Resource not found and build mode disabled");
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
            // Spawn a task to handle the websocket connection.
            tokio::spawn(async move {
                if let Err(e) = endpoint_ws(&uri, websocket, config).await {
                    log::trace!("[WS] Error in websocket connection: {}", e);
                }
            });

            // Return the response so the spawned future can continue.
            Ok(response)
        } else {
            log::error!("Unable to upgrade connection");
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
    websocket: HyperWebsocket,
    config: Arc<Mutex<Configuration>>,
) -> Result<(), Error> {
    let mut config = config.lock().await.to_owned();
    if let (Some(route), _parameter) =
        configuration::get_route(&config.routes, uri, &RouteMethod::WS)
    {
        let mut websocket = websocket.await?;

        let startup_files: Vec<Vec<u8>> = route
            .messages
            .par_iter()
            .filter(|c| c.kind == WsMessageType::Startup)
            .map(|c| data_loader::file_sync(c.location.as_str()))
            .filter(|c| c.is_ok())
            .map(|c| c.unwrap())
            .collect();
        for c in startup_files {
            websocket.send(Message::binary(c)).await?;
        }

        log::trace!("[WS] sent some messages");
        while let Some(message) = websocket.next().await {
            match message? {
                Message::Text(msg) => {
                    log::trace!("[WS] Received text message: {}", msg);
                    websocket
                        .send(Message::text("Thank you, come again."))
                        .await?;
                }
                Message::Binary(msg) => {
                    log::trace!("[WS] Received binary message: {:02X?}", msg);
                    websocket
                        .send(Message::binary(b"Thank you, come again.".to_vec()))
                        .await?;
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
                }
                Message::Frame(msg) => {
                    log::trace!("[WS] Received pong message: {:02X?}", msg);
                }
            }
        }
    } else {
        let route = builder::builder::build_ws(uri, websocket).await;
        config.routes.push(route);
    }

    Ok(())
}
