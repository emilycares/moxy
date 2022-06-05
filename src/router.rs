use std::{convert::Infallible, sync::Arc};

use hyper::{Body, Request, Response};
use tokio::sync::Mutex;

use crate::{
    builder,
    configuration::{self, Configuration, RouteMethod},
    data_loader,
};

pub async fn endpoint(
    req: Request<Body>,
    config_a: Arc<Mutex<Configuration>>,
) -> Result<Response<Body>, Infallible> {
    log::debug!("{}", req.uri());
    let configc = config_a.clone();
    let config = configc.lock().await.to_owned();
    let (route, parameter) =
        configuration::get_route(&config.routes, req.uri(), RouteMethod::from(req.method()));

    if route.is_some() {
        let data = data_loader::load(route.unwrap(), parameter);
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
        builder::build_response(config_a, req).await
    }
}
