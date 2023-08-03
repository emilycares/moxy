use hyper::{body::Bytes, HeaderMap};
use reqwest::Error;

use crate::{builder::core::ResourceData, configuration::RouteMethod};

/// Load data from external http source
pub async fn fetch_http(
    method: RouteMethod,
    url: impl reqwest::IntoUrl,
    body: impl Into<reqwest::Body>,
    header: HeaderMap,
    no_ssl_check: bool,
) -> Option<ResourceData> {
    let response = get_request(method.clone(), url, body, header, no_ssl_check).send().await;

    if let Ok(response) = response {
        return Some(ResourceData {
            method,
            headers: response.headers().clone(),
            code: response.status().as_u16(),
            payload: get_payload(response.bytes().await),
        });
    }

    None
}

/// Get request to Load data from external http source
pub fn get_request(
    method: RouteMethod,
    url: impl reqwest::IntoUrl,
    body: impl Into<reqwest::Body>,
    header: HeaderMap,
    no_ssl_check: bool,
) -> reqwest::RequestBuilder {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(no_ssl_check)
        .build()
        .unwrap_or_default();
    let mut req = client.request(method.to_owned().into(), url);

    req = req.headers(header);
    req = req.body(body);

    req
}

/// Convert request body to Vec<u8>
fn get_payload(bytes: Result<Bytes, Error>) -> Option<Vec<u8>> {
    bytes.map_or(None, |b| Some(b.to_vec()))
}
