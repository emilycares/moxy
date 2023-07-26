//! Returns a respomse to a given request.
use futures_util::StreamExt;
use futures_util::{sink::SinkExt, stream::FuturesUnordered};
use hyper::upgrade::Upgraded;
use hyper::HeaderMap;
use hyper_tungstenite::WebSocketStream;
use rayon::prelude::*;
use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use hyper_tungstenite::{tungstenite::Message, HyperWebsocket};
use tokio::sync::Mutex;

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
    uri: &str,
    method: hyper::Method,
    header: HeaderMap,
    body: hyper::Body,
) -> Result<Response<Body>, Infallible> {
    tracing::info!("{}", uri);
    let configc = config_a.clone();
    let mut config = configc.lock().await.to_owned();
    let (route, parameter) =
        configuration::get_route(&config.routes, uri, &RouteMethod::from(method.clone()));

    let Some(route) = route else {
         if config.build_mode == Some(BuildMode::Write) {
             return builder::core::build_response(config_a, uri, method, header, body).await
         } else {
             tracing::info!("Resource not found and build mode disabled");
             let response = Response::builder().status(404).body(Body::empty()).unwrap();
             return Ok(response);
         }
     };
    let data = data_loader::load(route, parameter);
    let Some(data) = data.await else {
        if let Some(x) = config.routes.iter().position(|c| c == route) {
            tracing::info!("Remove route because the file does not exist: {:?}", route);
            config.routes.remove(x);
        }

        if config.build_mode == Some(BuildMode::Write) {
            return builder::core::build_response(config_a, uri, method, header, body).await;
        } else {
            tracing::error!("Will build new route for missing file");
            let response = Response::builder().status(404).body(Body::empty()).unwrap();
            return Ok(response);
        }
    };
    let metadata = route.metadata.to_owned().unwrap_or_default();
    let mut resp_build = Response::builder().status(metadata.code).header(
        "content-type",
        get_content_type_with_fallback(metadata.header.clone(), route.resource.clone()),
    );

    for (key, value) in metadata.header.into_iter() {
        if let Some(key) = key {
            resp_build = resp_build.header(key, value);
        }
    }

    let response = resp_build.body(Body::from(data)).unwrap();

    Ok(response)
}

fn get_content_type_with_fallback(headers: HeaderMap, resource: Option<String>) -> String {
    return headers
        .get("content-type")
        .map(|v| v.to_str().unwrap_or_default())
        .map(|c| c.to_owned())
        .unwrap_or_else(|| {
            tracing::info!("Guessing content-type based on the file resource. Becaue it was not specified in the headers");
            return storage::get_content_type(resource
                                             .expect("save here because there will never be data without a resource",));
            });
}

async fn check_ws(
    request: Request<Body>,
    config: Arc<Mutex<Configuration>>,
) -> Result<Response<Body>, Infallible> {
    let uri = request.uri().path_and_query().unwrap().to_string();
    let method = request.method().clone();
    let headers = request.headers().clone();
    if hyper_tungstenite::is_upgrade_request(&request) {
        if let Ok((response, websocket)) = hyper_tungstenite::upgrade(request, None) {
            let restponse_status = response.status().as_u16().clone();
            // Spawn a task to handle the websocket connection.
            tokio::spawn(async move {
                if let Err(e) = endpoint_ws(
                    &uri,
                    Some(Metadata {
                        code: restponse_status,
                        header: headers,
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
        let header = request.headers().to_owned();
        let body = request.into_body();
        endpoint(config, &uri, method, header, body).await
    }
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
async fn endpoint_ws(
    uri: &str,
    metadata: Option<Metadata>,
    websocket: HyperWebsocket,
    config_a: Arc<Mutex<Configuration>>,
) -> Result<(), Error> {
    let config = config_a.clone();
    let mut config = config.lock().await.to_owned();
    let (Some(route), _parameter) = configuration::get_route(&config.routes, uri, &RouteMethod::WS) else {
      if config.build_mode == Some(BuildMode::Write) {
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
      return Ok(());
    };
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

    Ok(())
}

async fn send_ws_messages(
    mut rx: tokio::sync::mpsc::Receiver<Message>,
    mut websocket: WebSocketStream<Upgraded>,
) {
    loop {
        match rx.try_recv() {
            Ok(message) => match websocket.send(message).await {
                Ok(_) => tracing::trace!("Sent message"),
                Err(_) => tracing::error!("Failed to send message"),
            },
            Err(_) => ()
        }
    }
}
