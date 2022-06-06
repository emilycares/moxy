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
#[path = "./builder_test.rs"]
mod tests;
