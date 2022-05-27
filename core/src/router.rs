use std::{collections::HashMap, convert::Infallible};

use hyper::{Body, Request, Response};

use crate::{builder, configuration, data_loader};

pub async fn endpoint(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    log::debug!("{}", req.uri());
    let config = configuration::get_configuration();
    let routes = config.get_routes();
    let host = config.get_host();
    let (route, parameter) = configuration::get_route(&routes, req.uri(), req.method().as_str());

    if route.is_some() {
        let data = data_loader::load(&route.unwrap(), parameter);
        let response = Response::builder()
            .status(200)
            .body(Body::from(data))
            .unwrap();

        Ok(response)
    } else {
        log::debug!("{:?}", &req);

        let response = builder::fetch_request(
            req.method().clone(),
            builder::get_url(&req.uri(), host),
            None,
            HashMap::new(),
        )
        .await;

        match response {
            Some(response) => {
                if let Some(body) = response.payload {
                    //let body_content = match builder::get_body(body).await {
                        //Some(c) => c,
                        //None => "".to_string(),
                    //};
                    let response = Response::builder().status(200).body(Body::from(body.clone())).unwrap();

                    //const body_content = response.into_body();
                    let _res = tokio::fs::write("/tmp/testfile", body)
                    .await
                    .map_err(|e| eprint!("error writing file: {}", e));

                    Ok(response)
                } else {
                    let response = Response::builder().status(200).body(Body::empty()).unwrap();
                    Ok(response)
                }
            }
            None => todo!(),
        }
    }
}
