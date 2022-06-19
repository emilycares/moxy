use std::{collections::HashMap, convert::Infallible, sync::Arc};

use hyper::{Body, Request, Response};
use tokio::sync::Mutex;

use crate::configuration::{BuildMode, Configuration, RouteMethod};

use super::{request, storage};

/// The data structure that will contain all relevant data. To easily convert a request to a response
/// without doing a huge workaround.
pub struct ResourceData {
    /// HTTP method
    pub method: RouteMethod,
    /// HTTP heades
    pub headers: HashMap<String, String>,
    /// HTTP status code
    pub code: u16,
    /// HTTP body
    pub payload: Option<Vec<u8>>,
}

/// Handles unknown routes. It accomplishes that with creating HTTP request and saving the response
/// into a file. It also modifies the configuration in order to not call this function with the
/// same URL again.
pub async fn build_response(
    config_a: Arc<Mutex<Configuration>>,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let config_b = config_a.clone();
    let config = config_b.lock().await.to_owned();
    if let Some(build_mode) = &config.build_mode { 
        if let Some(remote) = &config.remote {
            let response = request::http::fetch_http(
                RouteMethod::from(req.method().clone()),
                request::util::get_url(req.uri(), remote),
                None,
                HashMap::new(),
            )
            .await;

            match response {
                Some(response) => {
                    if let Some(body) = response.payload {
                        if response.code != 404 && build_mode == &BuildMode::Write {
                            storage::save(
                                &response.method,
                                req.uri().path(),
                                body.clone(),
                                &response.headers,
                                config_a,
                            )
                            .await
                            .unwrap()
                        }

                        get_response(response.headers, response.code, Body::from(body))
                    } else {
                        get_response(response.headers, response.code, Body::empty())
                    }
                }
                None => {
                    log::error!("No response from endpoint");
                    let response = Response::builder().status(404).body(Body::empty()).unwrap();
                    Ok(response)
                }
            }
        } else {
            log::error!("Resource not found and no remove specified");
            let response = Response::builder().status(404).body(Body::empty()).unwrap();
            Ok(response)
        }
    } else {
        log::info!("Resource not found and build mode disabled");
        let response = Response::builder().status(404).body(Body::empty()).unwrap();
        Ok(response)
    }
}

/// Returns a respinse with headers and a code
pub fn get_response(headers: HashMap<String, String>, code: u16, body: Body) -> Result<Response<Body>, Infallible> {
    let mut response = Response::builder().status(code);

    for (key, value) in headers.into_iter() {
        response = response.header(key, value);
    }

    Ok(response.body(body).unwrap())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use hyper::Uri;

    use crate::{
        builder::request,
        configuration::{get_route, Route, RouteMethod},
    };

    #[tokio::test]
    async fn requrest_no_body() {
        let _reponse = request::http::fetch_http(
            RouteMethod::GET,
            "http://example.com".to_string(),
            None,
            HashMap::new(),
        )
        .await
        .unwrap();
    }

    #[test]
    fn get_route_should_not_find_entry_if_the_url_only_partialy_matches() {
        let routes = [Route {
            method: RouteMethod::GET,
            path: "/a".to_string(),
            resource: "".to_string(),
            messages: Vec::new()
        }];

        let uri = Uri::from_static("/a/test");

        let result = get_route(&routes, &uri, &RouteMethod::GET);

        assert_eq!(result, (None, None));
    }
}
