use cached::proc_macro::cached;
use hyper::Uri;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::ErrorKind;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub resource: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BuildMode {
    Read,
    Write,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Configuration {
    pub routes: Vec<Route>,
    pub host: Option<String>,
    pub remote: String,
    pub build_mode: Option<BuildMode>,
}

pub fn get_configuration() -> Configuration {
    load_configuration("./mockit.json".to_string())
}

pub fn get_route<'a>(
    routes: &'a [Route],
    uri: &'a Uri,
    method: &'a str,
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
fn load_configuration(loaction: String) -> Configuration {
    log::info!("Load Configuration: {}", loaction);
    let data = fs::read_to_string(&loaction).unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            File::create(loaction)
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
            remote: String::from("http://localhost"),
            build_mode: None,
        }
    })
}

#[cfg(test)]
#[path = "./configuration_test.rs"]
mod tests;
