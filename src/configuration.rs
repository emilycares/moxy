//! This contains the configuraton datastructures and the logic how to read and wirte it.

use hyper::{Method, Uri};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, io::ErrorKind, str::FromStr};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

/// This represents one route that can be navigated to
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Route {
    /// HTTP method
    pub method: RouteMethod,
    /// HTTP uri
    pub path: String,
    /// File storeage location
    pub resource: String,
    /// Data for WS
    pub messages: Vec<WsMessage>,
}

/// A WS message with controll when it has to be sent
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WsMessage {
    /// Type of time
    pub kind: WsMessageType,
    /// This will be contveted to WsMessageTime
    pub time: String,
    /// File storeage location
    pub location: String,
}

impl WsMessage {
    /// get parsed WsMessageTime
    pub fn get_time(&self) -> Option<WsMessageTime> {
        if let Ok(time) = self.time.parse::<WsMessageTime>() {
            return Some(time);
        }

        None
    }
}

/// Time units
#[derive(Debug, PartialEq)]
pub enum WsMessageTime {
    /// 5s
    Second(usize),
    /// 5m
    Minute(usize),
    /// 5h
    Hour(usize),
    /// 5sent ;After x messages sent messages
    Sent(usize),
    /// 5recived ;After x messages recived messages
    Recived(usize),
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
        } else if s.ends_with("recived") {
            let padding = 7;
            if let Ok(number) = parse_time_number(s, padding) {
                return Ok(Self::Recived(number));
            }
        }

        Err(1)
    }
}

impl Display for WsMessageTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsMessageTime::Second(t) => write!(f, "{}s", t),
            WsMessageTime::Minute(t) => write!(f, "{}m", t),
            WsMessageTime::Hour(t) => write!(f, "{}h", t),
            WsMessageTime::Sent(t) => write!(f, "{}sent", t),
            WsMessageTime::Recived(t) => write!(f, "{}recived", t),
        }
    }
}

/// This will take the number infront of a string
///
/// # Exaples
/// ```
/// let input = "3sent";
///
/// assert_eq!(parse_time_number(input, 3), 3);
/// ```
fn parse_time_number(number: &str, padding: usize) -> Result<usize, u8> {
    let padding = number.len() - padding;
    if let Some(number) = number.get(0..padding) {
        if let Ok(number) = number.parse::<usize>() {
            return Ok(number);
        } else {
            log::error!("Time is invalid format. Unable to parse number: {number}");
            return Err(2);
        }
    } else {
        log::error!("Time is invalid format. (To short)");
        Err(1)
    }
}

/// Type of time
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WsMessageType {
    /// When the ws connects
    Startup,
    /// Time after the ws connects
    After,
    /// Repeat
    Every,
}

/// This represents the http method that is used.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BuildMode {
    /// This specifies to not modifiy the filesystem or configuraion.
    Read,
    /// This enables to modify the filesystem and configuration when Needed.
    Write,
}

/// The datastructure for "moxy.json"
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Configuration {
    /// Specifies the port to run the http server.
    pub host: Option<String>,
    /// This url is called in `BuildMode::Write`
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
            .find(|c| c.resource.as_str() == resource && &c.method == method);

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

/// Loads the configuration from the filesystem.
pub async fn get_configuration() -> Configuration {
    load_configuration("./moxy.json".to_string()).await
}

/// Returns the route and an optional parameter.
///
/// The parameter can be used to milify the configuration when there is one dynamic part of the url
/// and file path.
///
/// | uri    | file       |
/// |--------|------------|
/// | /a.txt | ./db/a.txt |
/// | /b.txt | ./db/b.txt |
/// | /c.txt | ./db/c.txt |
/// | /d.txt | ./db/d.txt |
/// | /e.txt | ./db/e.txt |
///
/// In order to ceate configuration for this there would be a configuration entry for every uri.
/// But this can be simplified.
/// ``` json
/// {
///     "method": "GET",
///     "path": "/$$$.txt",
///     "resource": "./db/$$$.txt"
/// }
/// ```
pub fn get_route<'a>(
    routes: &'a [Route],
    uri: &'a Uri,
    method: &RouteMethod,
) -> (Option<&'a Route>, Option<&'a str>) {
    for i in routes.iter() {
        if i.method.eq(&method) {
            let index = &i.path.find("$$$");
            let path = &uri.path();

            if let Some(index) = index {
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

async fn load_configuration(loaction: String) -> Configuration {
    log::info!("Load Configuration: {}", loaction);
    match fs::read_to_string(&loaction).await {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|error| {
            log::error!("Could not load configuration file: {:?}", error);
            Configuration {
                routes: vec![],
                host: Some(String::from("127.0.0.1:8080")),
                remote: Some(String::from("http://localhost")),
                build_mode: None,
            }
        }),
        Err(e) => {
            let default_configuration = Configuration {
                routes: vec![],
                host: Some(String::from("127.0.0.1:8080")),
                remote: Some(String::from("http://localhost")),
                build_mode: Some(BuildMode::Write),
            };
            if e.kind() == ErrorKind::NotFound {
                save_configuration(default_configuration.clone())
                    .await
                    .unwrap();
            }

            default_configuration
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
    use hyper::Uri;

    use crate::configuration::{get_route, Route, RouteMethod, WsMessageTime};

    use super::Configuration;

    #[test]
    fn static_route() {
        let routes = vec![Route {
            method: RouteMethod::GET,
            path: "/api/test".to_string(),
            resource: "db/api/test.json".to_string(),
            messages: Vec::new(),
        }];
        let url = &"http://localhost:8080/api/test".parse::<Uri>().unwrap();
        let (result, parameter) = get_route(&routes, &url, &RouteMethod::GET);

        assert_eq!(result.unwrap().resource, routes[0].resource);
        assert_eq!(parameter, None);
    }

    #[test]
    fn configuration_get_route_should_find_no_route() {
        let configuration = Configuration {
            routes: vec![
                Route {
                    method: RouteMethod::GET,
                    path: "/a".to_string(),
                    resource: "somefile.txt".to_string(),
                    messages: Vec::new(),
                },
                Route {
                    method: RouteMethod::GET,
                    path: "/b".to_string(),
                    resource: "somefile.txt".to_string(),
                    messages: Vec::new(),
                },
                Route {
                    method: RouteMethod::GET,
                    path: "/c".to_string(),
                    resource: "somefile.txt".to_string(),
                    messages: Vec::new(),
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
                    path: "/a".to_string(),
                    resource: "somefile.txt".to_string(),
                    messages: Vec::new(),
                },
                Route {
                    method: RouteMethod::GET,
                    path: "/b".to_string(),
                    resource: "somefile.txt".to_string(),
                    messages: Vec::new(),
                },
                Route {
                    method: RouteMethod::GET,
                    path: "/c".to_string(),
                    resource: "somefile.txt".to_string(),
                    messages: Vec::new(),
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
                path: "/api/test/1/$$$.json".to_string(),
                resource: "db/api/1/$$$.json".to_string(),
                messages: Vec::new(),
            },
            Route {
                method: RouteMethod::GET,
                path: "/api/test/2/$$$.json".to_string(),
                resource: "db/api/2/$$$.json".to_string(),
                messages: Vec::new(),
            },
            Route {
                method: RouteMethod::GET,
                path: "/api/test/3/$$$.json".to_string(),
                resource: "db/api/3/$$$.json".to_string(),
                messages: Vec::new(),
            },
        ];

        assert_eq!(
            get_route(
                &routes,
                &"http://localhost:8080/api/test/1/abc.json"
                    .parse::<Uri>()
                    .unwrap(),
                &RouteMethod::GET
            )
            .0
            .unwrap()
            .resource,
            "db/api/1/$$$.json"
        );
        assert_eq!(
            get_route(
                &routes,
                &"http://localhost:8080/api/test/2/abc.json"
                    .parse::<Uri>()
                    .unwrap(),
                &RouteMethod::GET
            )
            .0
            .unwrap()
            .resource,
            "db/api/2/$$$.json"
        );
        assert_eq!(
            get_route(
                &routes,
                &"http://localhost:8080/api/test/3/abc.json"
                    .parse::<Uri>()
                    .unwrap(),
                &RouteMethod::GET
            )
            .0
            .unwrap()
            .resource,
            "db/api/3/$$$.json"
        );
    }

    #[test]
    fn dynamic_route_with_different_end() {
        let routes = vec![
            Route {
                method: RouteMethod::GET,
                path: "/api/test/$$$.txt".to_string(),
                resource: "db/api/$$$.txt".to_string(),
                messages: Vec::new(),
            },
            Route {
                method: RouteMethod::GET,
                path: "/api/test/$$$.json".to_string(),
                resource: "db/api/$$$.json".to_string(),
                messages: Vec::new(),
            },
        ];

        assert_eq!(
            get_route(
                &routes,
                &"http://localhost:8080/api/test/abc.txt"
                    .parse::<Uri>()
                    .unwrap(),
                &RouteMethod::GET
            )
            .0
            .unwrap()
            .resource,
            "db/api/$$$.txt"
        );
        assert_eq!(
            get_route(
                &routes,
                &"http://localhost:8080/api/test/abc.json"
                    .parse::<Uri>()
                    .unwrap(),
                &RouteMethod::GET
            )
            .0
            .unwrap()
            .resource,
            "db/api/$$$.json"
        );
    }

    #[test]
    fn dynamic_paramerter_end() {
        let routes = vec![Route {
            method: RouteMethod::GET,
            path: "/api/test/$$$".to_string(),
            resource: "db/api/$$$".to_string(),
            messages: Vec::new(),
        }];

        assert_eq!(
            get_route(
                &routes,
                &"http://localhost:8080/api/test/abc".parse::<Uri>().unwrap(),
                &RouteMethod::GET
            )
            .1
            .unwrap(),
            "abc"
        );
    }

    #[test]
    fn dynamic_paramerter_middle() {
        let routes = vec![Route {
            method: RouteMethod::GET,
            path: "/api/test/$$$.txt".to_string(),
            resource: "db/api/$$$.txt".to_string(),
            messages: Vec::new(),
        }];

        assert_eq!(
            get_route(
                &routes,
                &"http://localhost:8080/api/test/abc.txt"
                    .parse::<Uri>()
                    .unwrap(),
                &RouteMethod::GET
            )
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
            "3recived".parse::<WsMessageTime>().unwrap(),
            WsMessageTime::Recived(3)
        );
    }
}
