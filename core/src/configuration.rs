use cached::proc_macro::cached;
use hyper::{Method, Uri};
use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, str::FromStr};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Route {
    pub method: RouteMethod,
    pub path: String,
    pub resource: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum RouteMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl FromStr for RouteMethod {
    type Err = u8;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::GET),
            "HEAD" => Ok(Self::HEAD),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "CONNECT" => Ok(Self::CONNECT),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            "PATCH" => Ok(Self::PATCH),
            _ => Err(1),
        }
    }
}

impl From<Method> for RouteMethod {
    fn from(m: hyper::Method) -> Self {
        RouteMethod::from_str(m.as_str()).unwrap()
    }
}

impl From<&Method> for RouteMethod {
    fn from(m: &hyper::Method) -> Self {
        RouteMethod::from_str(m.as_str()).unwrap()
    }
}

impl Into<Method> for RouteMethod {
    fn into(self) -> Method {
        match self {
            RouteMethod::GET => Method::GET,
            RouteMethod::HEAD => Method::HEAD,
            RouteMethod::POST => Method::POST,
            RouteMethod::PUT => Method::PUT,
            RouteMethod::DELETE => Method::DELETE,
            RouteMethod::CONNECT => Method::CONNECT,
            RouteMethod::OPTIONS => Method::OPTIONS,
            RouteMethod::TRACE => Method::TRACE,
            RouteMethod::PATCH => Method::PATCH,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BuildMode {
    Read,
    Write,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Configuration {
    pub host: Option<String>,
    pub remote: Option<String>,
    pub build_mode: Option<BuildMode>,
    pub routes: Vec<Route>,
}

impl Configuration {
    pub fn has_route(&self, path: &str) -> bool {
      let matching_routes = self
            .routes
            .iter()
            .find(|c| c.path.as_str() == path);

      return matching_routes.is_some();
    }
}

pub async fn get_configuration() -> Configuration {
    load_configuration("./mockit.json".to_string()).await
}

pub fn get_route<'a>(
    routes: &'a [Route],
    uri: &'a Uri,
    method: RouteMethod,
) -> (Option<&'a Route>, Option<&'a str>) {
    for i in routes.iter() {
        if i.method.eq(&method) {
            let idx = &i.path.find("$$$");
            let path = &uri.path();

            if idx.is_some() {
                let index = &idx.unwrap();
                let match_before = &i.path[0..*index];

                if path.starts_with(&match_before) {
                    if index + 3 != i.path.len() {
                        let match_end = &i.path[index + 3..i.path.len()];

                        if path.ends_with(match_end) {
                            let sd = match_end.len();
                            return (Some(i), Some(&path[i.path.len() - 3 - sd..path.len() - sd]));
                        }
                    } else {
                        return (Some(i), Some(&path[i.path.len() - 3..path.len()]));
                    }
                }
            }
            if path.ends_with(&i.path) {
                return (Some(i), None);
            }
        }
    }

    (None, None)
}

#[cached]
async fn load_configuration(loaction: String) -> Configuration {
    log::info!("Load Configuration: {}", loaction);
    let data = fs::read_to_string(&loaction).await.unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            std::fs::File::create(loaction)
                .unwrap_or_else(|error| panic!("Could not open configuration file: {:?}", error));
        } else {
            log::info!("Could not open configuration file: {:?}", error);
        }
        "".to_string()
    });

    serde_json::from_str(&data).unwrap_or_else(|error| {
        log::error!("Could not load configuration file: {:?}", error);
        Configuration {
            routes: vec![],
            host: Some(String::from("127.0.0.1:8080")),
            remote: Some(String::from("http://localhost")),
            build_mode: None,
        }
    })
}

pub async fn save_configuration(configuration: Configuration) -> Result<(), std::io::Error> {
    let config: String = serde_json::to_string_pretty(&configuration)?;
    let mut file = File::create("./mockit.json").await?;

    file.write_all(config.as_bytes()).await?;

    file.flush().await?;

    Ok(())
}

#[cfg(test)]
#[path = "./configuration_test.rs"]
mod tests;
