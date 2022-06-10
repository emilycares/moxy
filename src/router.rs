//! Returns a respomse to a given request.
use std::{convert::Infallible, sync::Arc};

use hyper::{Body, Request, Response};
use tokio::sync::Mutex;

use crate::{
    builder,
    configuration::{self, Configuration, RouteMethod},
    data_loader,
};

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
