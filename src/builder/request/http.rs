use hyper::{body::Bytes, HeaderMap};
use reqwest::Error;

use crate::{builder::core::ResourceData, configuration::RouteMethod};

use super::util::header_map_to_hash_map;

/// Load data from external http source
pub async fn fetch_http(
    method: RouteMethod,
    url: impl reqwest::IntoUrl,
    body: impl Into<reqwest::Body>,
    header: HeaderMap,
) -> Option<ResourceData> {
    let response = get_request(method.clone(), url, body, header).send().await;

    if let Ok(response) = response {
        return Some(ResourceData {
            method,
            headers: header_map_to_hash_map(response.headers()),
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
) -> reqwest::RequestBuilder {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
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
