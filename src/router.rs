use std::convert::Infallible;

use hyper::{Body, Request, Response};

use crate::{configuration, data_loader};

pub(crate) async fn endpoint(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{}", req.uri());
    let routes = configuration::get_routes();
    let (route, parameter) = configuration::get_route(&routes, req.uri(), req.method().as_str());

    if route.is_some() {
        let data = data_loader::load(&route.unwrap(), parameter);
        let response = Response::builder()
            .status(200)
            .body(Body::from(data))
            .unwrap();

        Ok(response)
    } else {
        let response = Response::builder()
            .status(404)
            .body(Body::from(format!(
                "Could not load resource {}",
                &req.uri()
            )))
            .unwrap();
        Ok(response)
    }
}
