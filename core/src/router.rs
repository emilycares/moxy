use std::convert::Infallible;

use hyper::{Body, Request, Response};

use crate::{builder::fetch_request, configuration, data_loader};

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
        println!("{:?}", &req);

        let body = fetch_request(req, String::from("https://smashmc.eu")).await;

        match body {
            Some(body) => {
                let response = Response::builder().status(200).body(body).unwrap();

                Ok(response)
            }
            None => todo!(),
        }
    }
}
