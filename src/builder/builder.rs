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
            let response = request::fetch_request(
                RouteMethod::from(req.method().clone()),
                request::get_url(req.uri(), remote),
                None,
                HashMap::new(),
            )
            .await;

            match response {
                Some(response) => {
                    if let Some(body) = response.payload {
                        let client_response = Response::builder()
                            .status(response.code)
                            .body(Body::from(body.clone()))
                            .unwrap();

                        if response.code != 404 && build_mode == &BuildMode::Write {
                            storage::save(response.method, req.uri().path(), body, config_a)
                                .await
                                .unwrap()
                        }

                        Ok(client_response)
                    } else {
                        let response = Response::builder()
                            .status(response.code)
                            .body(Body::empty())
                            .unwrap();
                        Ok(response)
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use hyper::Uri;

    use crate::{
        builder::{request, storage::get_save_path},
        configuration::{get_route, Route, RouteMethod},
    };

    #[tokio::test]
    async fn requrest_no_body() {
        let _reponse = request::fetch_request(
            RouteMethod::GET,
            "http://example.com".to_string(),
            None,
            HashMap::new(),
        )
        .await
        .unwrap();
    }

    #[test]
    fn get_save_path_should_start_with_db() {
        let path = get_save_path("/index.html");

        assert!(&path.starts_with("./db"));
    }

    #[test]
    fn get_save_path_should_add_index_if_folder() {
        let path = get_save_path("/");

        assert!(&path.ends_with("/index"));
    }

    #[test]
    fn get_route_should_not_find_entry_if_the_url_only_partialy_matches() {
        let routes = [Route {
            method: RouteMethod::GET,
            path: "/a".to_string(),
            resource: "".to_string(),
        }];

        let uri = Uri::from_static("/a/test");

        let result = get_route(&routes, &uri, &RouteMethod::GET);

        assert_eq!(result, (None, None));
    }
}
