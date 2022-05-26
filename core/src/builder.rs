use std::collections::HashMap;

use hyper::{HeaderMap, Uri, Method, Body};
use reqwest::Error;


#[derive(Debug, Clone)]
pub struct ResourceData {
    headers: HashMap<String, String>,
    code: u16,
    pub payload: Option<String>,
}


pub fn get_url(uri: &Uri, host: String) -> String {
    host + &uri.to_string()
}

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

pub async fn fetch_request(
    method: Method,
    url: String,
    body: Option<String>,
    header: HashMap<String, String>,
) -> Option<ResourceData> {
    let client = reqwest::Client::new();
    let mut req = client.request(method, url);

    //if let Ok(header) = header.try_from() {
    //req.headers(header);
    //}

    if let Some(body) = body {
        req = req.body(body);
    }

    let response = req.send().await;

    if let Ok(response) = response {
        return Some(ResourceData {
            headers: header_map_to_hash_map(&response.headers()),
            code: response.status().as_u16(),
            payload: get_payload(&response.text().await),
        });
    }

    None
}

fn get_payload(text: &Result<String, Error>) -> Option<String> {
    match text {
        Ok(p) => Some(p.to_string()),
        Err(_) => None,
    }
}

pub async fn get_body(body: Body) -> Option<String> {
    // size check

    // return result
    match hyper::body::to_bytes(body).await {
        Ok(bytes) => match String::from_utf8(bytes.to_vec()) {
            Ok(content) => Some(content),
            Err(err) => {
                println!("Unable to parse body content as utf8, {:?}", err);
                None
            }
        },
        Err(err) => {
            println!("Unable to get bytes form request, {:?}", err);
            None
        }
    }
}


#[cfg(test)]
#[path = "./builder_test.rs"]
mod tests;
