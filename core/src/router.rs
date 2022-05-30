use std::{convert::Infallible, sync::Arc};

use hyper::{Body, Request, Response};
use tokio::sync::Mutex;

use crate::{
    builder,
    configuration::{self, Configuration},
    data_loader,
};

pub async fn endpoint(
    req: Request<Body>,
    _config: Arc<Mutex<Configuration>>,
) -> Result<Response<Body>, Infallible> {
    log::debug!("{}", req.uri());
    let config = configuration::get_configuration();
    let (route, parameter) =
        configuration::get_route(&config.routes, req.uri(), req.method().as_str());

    if route.is_some() {
        let data = data_loader::load(route.unwrap(), parameter);
        let response = Response::builder()
            .status(200)
            .body(Body::from(data))
            .unwrap();

        Ok(response)
    } else {
        builder::build_response(req).await
    }
}
