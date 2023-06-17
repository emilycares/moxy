use std::{collections::HashMap, convert::Infallible, sync::Arc};

use hyper::{Body, Response};
use tokio::sync::Mutex;

use crate::configuration::{BuildMode, Configuration, Metadata, RouteMethod};

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
    uri: &hyper::Uri,
    method: &hyper::Method,
) -> Result<Response<Body>, Infallible> {
    let config_b = config_a.clone();
    let config = config_b.lock().await.to_owned();
    if let Some(build_mode) = &config.build_mode {
        if let Some(remote) = &config.remote {
            let response = request::http::fetch_http(
                RouteMethod::from(method),
                request::util::get_url(uri, remote),
                None,
                HashMap::new(),
            )
            .await;

            if let Some(response) = response {
                if let Some(body) = response.payload {
                    if response.code != 404 && build_mode == &BuildMode::Write {
                        storage::save(
                            &response.method,
                            uri.path(),
                            Some(Metadata {
                                code: response.code,
                                headers: response.headers.clone(),
                            }),
                            body.clone(),
                            config_a,
                        )
                        .await
                        .unwrap();
                    }

                    get_response(response.headers, response.code, Body::from(body))
                } else {
                    get_response(response.headers, response.code, Body::empty())
                }
            } else {
                                tracing::error!("No response from endpoint");
                                let response = Response::builder().status(404).body(Body::empty()).unwrap();
                                Ok(response)
                            }
        } else {
            tracing::error!("Resource not found and no remove specified");
            let response = Response::builder().status(404).body(Body::empty()).unwrap();
            Ok(response)
        }
    } else {
        tracing::info!("Resource not found and build mode disabled");
        let response = Response::builder().status(404).body(Body::empty()).unwrap();
        Ok(response)
    }
}

/// Returns a respinse with headers and a code
pub fn get_response(
    headers: HashMap<String, String>,
    code: u16,
    body: Body,
) -> Result<Response<Body>, Infallible> {
    let mut response = Response::builder().status(code);

    for (key, value) in headers.into_iter() {
        response = response.header(key, value);
    }

    Ok(response.body(body).unwrap())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{builder::request, configuration::RouteMethod};

    #[tokio::test]
    async fn request_no_body() {
        let _response = request::http::fetch_http(
            RouteMethod::GET,
            "http://example.com".to_string(),
            None,
            HashMap::new(),
        )
        .await
        .unwrap();
    }
}
