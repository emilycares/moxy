//! This contains the configuration datastructures and the logic how to read and write it.

use hyper::{Method, HeaderMap};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    convert::TryInto, fmt::Display, io::ErrorKind, str::FromStr,
    time::Duration,
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

/// This represents one route that can be navigated to
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Route {
    /// HTTP method
    pub method: RouteMethod,
    /// HTTP uri
    pub path: String,
    /// Response metadata
    pub metadata: Option<Metadata>,
    /// File storage location
    #[serde(default)]
    pub resource: Option<String>,
    /// Data for WS
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<WsMessage>,
}

/// Metadata for the response
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    /// HTTP status code
    pub code: u16,
    /// HTTP headers
    #[serde(with = "http_serde::header_map")]
    pub header: HeaderMap,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            code: 200,
            header: HeaderMap::new()
        }
    }
}

/// A WS message with control when it has to be sent
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WsMessage {
    /// Type of time
    pub kind: WsMessageType,
    /// This will be contveted to WsMessageTime
    #[serde(default)]
    pub time: Option<String>,
    /// The message type that has to be sent
    pub message_type: WsMessagType,
    /// File storage location
    pub location: String,
}

impl WsMessage {
    /// get parsed WsMessageTime
    pub fn get_time(&self) -> Option<Duration> {
        if let Some(time) = &self.time {
            if let Ok(time) = time.parse::<WsMessageTime>() {
                return Some(Duration::from(time));
            }
        }

        None
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
/// The type of the websocket message.
pub enum WsMessagType {
    /// A text WebSocket message
    Text,
    /// A binary WebSocket message
    Binary,
    /// A ping message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Ping,
    /// A pong message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Pong,
    /// A close message with the optional close frame.
    Close,
    /// Raw frame. Note, that you're not going to get this value while reading the message.
    Frame,
}

/// Time units
#[derive(Debug, PartialEq, Eq)]
pub enum WsMessageTime {
    /// 5s
    Second(usize),
    /// 5m
    Minute(usize),
    /// 5h
    Hour(usize),
    /// 5sent ;After x messages sent messages
    Sent(usize),
    /// 5received ;After x messages received messages
    Received(usize),
}

impl FromStr for WsMessageTime {
    type Err = u8;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with('s') {
            let padding = 1;
            if let Ok(number) = parse_time_number(s, padding) {
                return Ok(Self::Second(number));
            }
        } else if s.ends_with('m') {
            let padding = 1;
            if let Ok(number) = parse_time_number(s, padding) {
                return Ok(Self::Minute(number));
            }
        } else if s.ends_with('h') {
            let padding = 1;
            if let Ok(number) = parse_time_number(s, padding) {
                return Ok(Self::Hour(number));
            }
        } else if s.ends_with("sent") {
            let padding = 4;
            if let Ok(number) = parse_time_number(s, padding) {
                return Ok(Self::Sent(number));
            }
        } else if s.ends_with("received") {
            let padding = 8;
            if let Ok(number) = parse_time_number(s, padding) {
                return Ok(Self::Received(number));
            }
        }

        Err(1)
    }
}

impl Display for WsMessageTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Second(t) => write!(f, "{t}s"),
            Self::Minute(t) => write!(f, "{t}m"),
            Self::Hour(t) => write!(f, "{t}h"),
            Self::Sent(t) => write!(f, "{t}sent"),
            Self::Received(t) => write!(f, "{t}received"),
        }
    }
}

impl From<WsMessageTime> for Duration {
    fn from(wt: WsMessageTime) -> Self {
        let seconds = match wt {
            WsMessageTime::Second(s) => s,
            WsMessageTime::Minute(m) => 60 * m,
            WsMessageTime::Hour(h) => 60 * 60 * h,
            WsMessageTime::Sent(_) => 1,
            WsMessageTime::Received(_) => 1,
        };

        Duration::from_secs(seconds.try_into().unwrap())
    }
}

/// This will take the number infront of a string
///
/// # Examples
/// ```
/// let input = "3sent";
///
/// assert_eq!(parse_time_number(input, 3), 3);
/// ```
fn parse_time_number(number: &str, padding: usize) -> Result<usize, u8> {
    let padding = number.len() - padding;
    if let Some(number) = number.get(0..padding) {
        if let Ok(number) = number.parse::<usize>() {
            Ok(number)
        } else {
            tracing::error!("Time is invalid format. Unable to parse number: {number}");
            Err(2)
        }
    } else {
        tracing::error!("Time is invalid format. (To short)");
        Err(1)
    }
}

/// Type of time
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum WsMessageType {
    /// When the ws connects
    Startup,
    /// Time after the ws connects
    After,
    /// Repeat
    Every,
}

/// This represents the http method that is used.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RouteMethod {
    /// HTTP GET
    GET,
    /// HTTP HEAD
    HEAD,
    /// HTTP POST
    POST,
    /// HTTP PUT
    PUT,
    /// HTTP DELETE
    DELETE,
    /// HTTP CONNECT
    CONNECT,
    /// HTTP OPTIONS
    OPTIONS,
    /// HTTP TRACE
    TRACE,
    /// HTTP PATCH
    PATCH,
    /// Websocket
    WS,
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
            "WS" => Ok(Self::WS),
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

impl From<RouteMethod> for Method {
    fn from(route_method: RouteMethod) -> Self {
        match route_method {
            RouteMethod::GET => Method::GET,
            RouteMethod::HEAD => Method::HEAD,
            RouteMethod::POST => Method::POST,
            RouteMethod::PUT => Method::PUT,
            RouteMethod::DELETE => Method::DELETE,
            RouteMethod::CONNECT => Method::CONNECT,
            RouteMethod::OPTIONS => Method::OPTIONS,
            RouteMethod::TRACE => Method::TRACE,
            RouteMethod::PATCH => Method::PATCH,
            RouteMethod::WS => unreachable!(),
        }
    }
}

/// The configuration setting for `build_mode`
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BuildMode {
    /// This specifies to not modify the filesystem or configuration.
    Read,
    /// This enables to modify the filesystem and configuration when Needed.
    Write,
}

/// The datastructure for "moxy.json"
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Configuration {
    /// Specifies the port to run the http server.
    pub host: Option<String>,
    /// This url is called when build_mode is set to `BuildMode::Write`
    pub remote: Option<String>,
    /// `BuildMode`
    pub build_mode: Option<BuildMode>,
    /// A list of all available routes.
    pub routes: Vec<Route>,
}

impl Configuration {
    /// Checks if there is an existing route based on the path and method
    pub fn get_route(&self, path: &str, method: &RouteMethod) -> Option<&Route> {
        let matching_routes = self
            .routes
            .iter()
            .find(|c| c.path.as_str() == path && &c.method == method);

        matching_routes
    }

    /// Checks if there is an existing route based on the resource and method
    pub fn get_route_by_resource_mut(
        &mut self,
        resource: &str,
        method: &RouteMethod,
    ) -> Option<&mut Route> {
        let matching_routes = self
            .routes
            .iter_mut()
            .filter(|c| c.resource.is_some())
            .find(|c| c.resource.as_ref().unwrap().as_str() == resource && &c.method == method);

        matching_routes
    }

    /// Checks if there is an existing route based on the path and method
    pub fn get_route_by_path_mut(
        &mut self,
        path: &str,
        method: &RouteMethod,
    ) -> Option<&mut Route> {
        let matching_routes = self
            .routes
            .iter_mut()
            .find(|c| c.path.as_str() == path && &c.method == method);

        matching_routes
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            host: Some(String::from("127.0.0.1:8080")),
            remote: Some(String::from("http://localhost")),
            build_mode: Some(BuildMode::Read),
            routes: vec![],
        }
    }
}

/// Loads the configuration from the filesystem.
pub async fn get_configuration() -> Configuration {
    load_configuration("./moxy.json").await
}

/// Returns the route and an optional parameter.
///
/// The parameter can be used to milify the configuration
/// when there is one dynamic part of the url and file path.
///
/// | uri    | file       |
/// |--------|------------|
/// | /a.txt | ./db/a.txt |
/// | /b.txt | ./db/b.txt |
/// | /c.txt | ./db/c.txt |
/// | /d.txt | ./db/d.txt |
/// | /e.txt | ./db/e.txt |
///
/// In order to create configuration for this there would be a configuration
/// entry for every uri. But this can be simplified.
/// ``` json
/// {
///     "method": "GET",
///     "path": "/$$$.txt",
///     "resource": "./db/$$$.txt"
/// }
/// ```
pub fn get_route<'a>(
    routes: &'a [Route],
    uri: &'a str,
    method: &RouteMethod,
) -> (Option<&'a Route>, Option<&'a str>) {
    for i in routes.iter() {
        if i.method.eq(method) {
            let index = &i.path.find("$$$");

            if let Some(index) = index {
                let match_before = &i.path[0..*index];

                if uri.starts_with(match_before) {
                    if index + 3 != i.path.len() {
                        let match_end = &i.path[index + 3..i.path.len()];

                        if uri.ends_with(match_end) {
                            let sd = match_end.len();
                            return (Some(i), Some(&uri[i.path.len() - 3 - sd..uri.len() - sd]));
                        }
                    } else {
                        return (Some(i), Some(&uri[i.path.len() - 3..uri.len()]));
                    }
                }
            }
            if uri.ends_with(&i.path) {
                return (Some(i), None);
            }
        }
    }

    (None, None)
}

async fn load_configuration(location: &str) -> Configuration {
    tracing::info!("Load Configuration: {}", location);
    match fs::read_to_string(&location).await {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|error| {
            tracing::error!("Could not load configuration file: {:?}", error);
            Configuration::default()
        }),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                save_configuration(Configuration::default()).await.unwrap();
            }

            Configuration::default()
        }
    }
}

/// Save configuration to filesystem
pub async fn save_configuration(configuration: Configuration) -> Result<(), std::io::Error> {
    let config: String = serde_json::to_string_pretty(&configuration)?;
    let mut file = File::create("./moxy.json").await?;

    file.write_all(config.as_bytes()).await?;

    file.flush().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::configuration::{get_route, Route, RouteMethod, WsMessageTime};

    use super::Configuration;

    #[test]
    fn static_route() {
        let routes = vec![Route {
            method: RouteMethod::GET,
            metadata: Option::None,
            path: "/api/test".to_string(),
            resource: Some("db/api/test.json".to_string()),
            messages: vec![],
        }];
        let url = "http://localhost:8080/api/test";
        let (result, parameter) = get_route(&routes, url, &RouteMethod::GET);

        assert_eq!(result.unwrap().resource, routes[0].resource);
        assert_eq!(parameter, None);
    }

    #[test]
    fn configuration_get_route_should_find_no_route() {
        let configuration = Configuration {
            routes: vec![
                Route {
                    method: RouteMethod::GET,
                    metadata: Option::None,
                    path: "/a".to_string(),
                    resource: Some("somefile.txt".to_string()),
                    messages: vec![],
                },
                Route {
                    method: RouteMethod::GET,
                    metadata: Option::None,
                    path: "/b".to_string(),
                    resource: Some("somefile.txt".to_string()),
                    messages: vec![],
                },
                Route {
                    method: RouteMethod::GET,
                    metadata: Option::None,
                    path: "/c".to_string(),
                    resource: Some("somefile.txt".to_string()),
                    messages: vec![],
                },
            ],
            host: None,
            remote: None,
            build_mode: None,
        };

        assert!(!configuration.get_route("/abc", &RouteMethod::GET).is_some());
    }

    #[test]
    fn configuration_get_route_should_find_route() {
        let configuration = Configuration {
            routes: vec![
                Route {
                    method: RouteMethod::GET,
                    metadata: Option::None,
                    path: "/a".to_string(),
                    resource: Some("somefile.txt".to_string()),
                    messages: vec![],
                },
                Route {
                    method: RouteMethod::GET,
                    metadata: Option::None,
                    path: "/b".to_string(),
                    resource: Some("somefile.txt".to_string()),
                    messages: vec![],
                },
                Route {
                    method: RouteMethod::GET,
                    metadata: Option::None,
                    path: "/c".to_string(),
                    resource: Some("somefile.txt".to_string()),
                    messages: vec![],
                },
            ],
            host: None,
            remote: None,
            build_mode: None,
        };

        assert!(configuration.get_route("/a", &RouteMethod::GET).is_some());
        assert!(configuration.get_route("/b", &RouteMethod::GET).is_some());
        assert!(configuration.get_route("/c", &RouteMethod::GET).is_some());
    }

    #[test]
    fn dynamic_route_with_different_start() {
        let routes = vec![
            Route {
                method: RouteMethod::GET,
                metadata: Option::None,
                path: "/api/test/1/$$$.json".to_string(),
                resource: Some("db/api/1/$$$.json".to_string()),
                messages: vec![],
            },
            Route {
                method: RouteMethod::GET,
                metadata: Option::None,
                path: "/api/test/2/$$$.json".to_string(),
                resource: Some("db/api/2/$$$.json".to_string()),
                messages: vec![],
            },
            Route {
                method: RouteMethod::GET,
                metadata: Option::None,
                path: "/api/test/3/$$$.json".to_string(),
                resource: Some("db/api/3/$$$.json".to_string()),
                messages: vec![],
            },
        ];

        assert_eq!(
            get_route(&routes, "/api/test/1/abc.json", &RouteMethod::GET)
                .0
                .unwrap()
                .resource
                .as_ref()
                .unwrap(),
            "db/api/1/$$$.json"
        );
        assert_eq!(
            get_route(&routes, "/api/test/2/abc.json", &RouteMethod::GET)
                .0
                .unwrap()
                .resource
                .as_ref()
                .unwrap(),
            "db/api/2/$$$.json"
        );
        assert_eq!(
            get_route(&routes, "/api/test/3/abc.json", &RouteMethod::GET)
                .0
                .unwrap()
                .resource
                .as_ref()
                .unwrap(),
            "db/api/3/$$$.json"
        );
    }

    #[test]
    fn dynamic_route_with_different_end() {
        let routes = vec![
            Route {
                method: RouteMethod::GET,
                metadata: Option::None,
                path: "/api/test/$$$.txt".to_string(),
                resource: Some("db/api/$$$.txt".to_string()),
                messages: vec![],
            },
            Route {
                method: RouteMethod::GET,
                metadata: Option::None,
                path: "/api/test/$$$.json".to_string(),
                resource: Some("db/api/$$$.json".to_string()),
                messages: vec![],
            },
        ];

        assert_eq!(
            get_route(&routes, "/api/test/abc.txt", &RouteMethod::GET)
                .0
                .unwrap()
                .resource
                .as_ref()
                .unwrap(),
            "db/api/$$$.txt"
        );
        assert_eq!(
            get_route(&routes, "/api/test/abc.json", &RouteMethod::GET)
                .0
                .unwrap()
                .resource
                .as_ref()
                .unwrap(),
            "db/api/$$$.json"
        );
    }

    #[test]
    fn dynamic_paramerter_end() {
        let routes = vec![Route {
            method: RouteMethod::GET,
            metadata: Option::None,
            path: "/api/test/$$$".to_string(),
            resource: Some("db/api/$$$".to_string()),
            messages: vec![],
        }];

        assert_eq!(
            get_route(&routes, "/api/test/abc", &RouteMethod::GET)
                .1
                .unwrap(),
            "abc"
        );
    }

    #[test]
    fn dynamic_paramerter_middle() {
        let routes = vec![Route {
            method: RouteMethod::GET,
            metadata: Option::None,
            path: "/api/test/$$$.txt".to_string(),
            resource: Some("db/api/$$$.txt".to_string()),
            messages: vec![],
        }];

        assert_eq!(
            get_route(&routes, "/api/test/abc.txt", &RouteMethod::GET)
                .1
                .unwrap(),
            "abc"
        );
    }

    #[test]
    fn parse_ws_message_time() {
        assert_eq!(
            "3s".parse::<WsMessageTime>().unwrap(),
            WsMessageTime::Second(3)
        );
        assert_eq!(
            "3m".parse::<WsMessageTime>().unwrap(),
            WsMessageTime::Minute(3)
        );
        assert_eq!(
            "3h".parse::<WsMessageTime>().unwrap(),
            WsMessageTime::Hour(3)
        );
        assert_eq!(
            "3sent".parse::<WsMessageTime>().unwrap(),
            WsMessageTime::Sent(3)
        );
        assert_eq!(
            "3received".parse::<WsMessageTime>().unwrap(),
            WsMessageTime::Received(3)
        );
    }

    #[test]
    fn get_route_should_not_find_entry_if_the_url_only_partially_matches() {
        let routes = [Route {
            method: RouteMethod::GET,
            metadata: Option::None,
            path: "/a".to_string(),
            resource: Some("".to_string()),
            messages: vec![],
        }];

        let uri = "/a/test";

        let result = get_route(&routes, &uri, &RouteMethod::GET);

        assert_eq!(result, (None, None));
    }
}
