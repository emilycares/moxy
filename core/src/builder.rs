use std::{collections::HashMap, convert::Infallible, str::FromStr, sync::Arc};

use hyper::{
    body::Bytes,
    header::{HeaderName, HeaderValue},
    Body, HeaderMap, Request, Response, Uri,
};
use reqwest::Error;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    sync::Mutex,
};

use crate::configuration::{self, BuildMode, Configuration, Route, RouteMethod};

pub struct ResourceData {
    pub method: RouteMethod,
    pub headers: HashMap<String, String>,
    pub code: u16,
    pub payload: Option<Vec<u8>>,
}

pub async fn build_response(
    config_a: Arc<Mutex<Configuration>>,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let config_b = config_a.clone();
    let config = config_b.lock().await.to_owned();
    if let Some(build_mode) = &config.build_mode {
        if let Some(remote) = &config.remote {
            let response = fetch_request(
                RouteMethod::from(req.method().clone()),
                get_url(req.uri(), remote),
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
                            save(response.method, req.uri().path(), body, config_a)
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

async fn save(
    method: RouteMethod,
    uri: &str,
    body: Vec<u8>,
    config: Arc<Mutex<Configuration>>,
) -> Result<(), std::io::Error> {
    let path = get_save_path(uri);
    let mut config = config.lock().await;
    if !config.has_route(&path) {
        let route = Route {
            method,
            resource: path.clone(),
            path: uri.to_owned(),
        };
        log::info!("Save route: {:?}", route);

        config.routes.push(route);

        save_file(path.as_str(), body).await?;
        configuration::save_configuration(config.to_owned()).await?;
    }

    Ok(())
}

async fn save_file(location: &str, body: Vec<u8>) -> Result<(), std::io::Error> {
    let folders: String = if let Some(index) = location.rfind('/') {
        location[0..index].to_owned()
    } else {
        location.to_owned()
    };
    log::trace!("Create folders: {}", folders);
    fs::create_dir_all(folders).await?;

    let mut file = File::create(location).await?;
    file.write_all(&body).await?;

    Ok(())
}

pub fn get_save_path(uri: &str) -> String {
    let mut path = "./db".to_owned() + uri;

    if path.ends_with('/') {
        path += "index"
    }

    path
}

pub fn get_url(uri: &Uri, host: &String) -> String {
    host.to_owned() + &uri.to_string()
}

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

fn get_payload(bytes: Result<Bytes, Error>) -> Option<Vec<u8>> {
    match bytes {
        Ok(bytes) => Some(bytes.to_vec()),
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
                log::error!("Unable to parse body content as utf8, {:?}", err);
                None
            }
        },
        Err(err) => {
            log::error!("Unable to get bytes form request, {:?}", err);
            None
        }
    }
}

#[cfg(test)]
#[path = "./builder_test.rs"]
mod tests;
