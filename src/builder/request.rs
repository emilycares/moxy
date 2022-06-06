use std::{collections::HashMap, str::FromStr};

use hyper::{Uri, HeaderMap, header::{HeaderName, HeaderValue}, body::Bytes};
use reqwest::Error;

use crate::configuration::RouteMethod;

use super::builder::ResourceData;

/// Get url with the new host
pub fn get_url(uri: &Uri, host: &String) -> String {
    host.to_owned() + &uri.to_string()
}

/// Convert HashMap to HeaderMap
fn hash_map_to_header_map(map: HashMap<String, String>) -> HeaderMap {
    let keys = map.keys();
    let mut out = HeaderMap::new();

    for key in keys {
        if let Some(value) = map.get(key) {
            let key = HeaderName::from_str(key.to_owned().as_str());
            if let Ok(key) = key {
                let value = HeaderValue::from_str(value.to_owned().as_str());
                if let Ok(value) = value {
                    log::info!("Some header");
                    out.insert(key, value);
                }
            }
        }
    }

    out
}

/// Convert Headermap to HashMap
fn header_map_to_hash_map(header: &HeaderMap) -> HashMap<String, String> {
    let keys = header.keys();
    let mut out: HashMap<String, String> = HashMap::new();

    for key in keys {
        if let Some(value) = header.get(key) {
            if let Ok(value) = value.to_owned().to_str() {
                out.insert(key.to_string(), value.to_string());
            }
        }
    }

    out
}

/// Load data from external http source
pub async fn fetch_request(
    method: RouteMethod,
    url: String,
    body: Option<String>,
    header: HashMap<String, String>,
) -> Option<ResourceData> {
    let client = reqwest::Client::new();
    let mut req = client.request(method.clone().into(), url);

    req = req.headers(hash_map_to_header_map(header));

    if let Some(body) = body {
        req = req.body(body);
    }

    let response = req.send().await;

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

/// Convert request body to Vec<u8>
fn get_payload(bytes: Result<Bytes, Error>) -> Option<Vec<u8>> {
    match bytes {
        Ok(bytes) => Some(bytes.to_vec()),
        Err(_) => None,
    }
}
