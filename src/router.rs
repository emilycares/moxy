//! Returns a respomse to a given request.
use std::{convert::Infallible, sync::Arc, net::SocketAddr};

use hyper::{
    service::{make_service_fn, service_fn},
    Server, Body, Request, Response,
};
use tokio::sync::Mutex;

use crate::{
    builder,
    configuration::{self, Configuration, RouteMethod},
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
                    async move { endpoint(req, config).await }
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
pub async fn endpoint(
    req: Request<Body>,
    config_a: Arc<Mutex<Configuration>>,
) -> Result<Response<Body>, Infallible> {
    log::info!("{}", req.uri());
    let configc = config_a.clone();
    let config = configc.lock().await.to_owned();
    let (route, parameter) =
        configuration::get_route(&config.routes, req.uri(), &RouteMethod::from(req.method()));

    if let Some(route) = route {
        let data = data_loader::load(route, parameter);
        if let Some(data) = data.await {
            let response = Response::builder()
                .status(200)
                .body(Body::from(data))
                .unwrap();

            Ok(response)
        } else {
            log::error!("Resource not found and Requested file was not found on file system");
            let response = Response::builder().status(404).body(Body::empty()).unwrap();
            Ok(response)
        }
    } else {
        builder::builder::build_response(config_a, req).await
    }
}
