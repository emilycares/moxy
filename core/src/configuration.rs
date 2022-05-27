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

#[derive(Serialize, Deserialize, Clone)]
pub struct Configuration {
    routes: Vec<Route>,
    host: String,
}

impl Configuration {
    pub fn get_routes(&self) -> Vec<Route> {
        self.clone().routes
    }
    pub fn get_host(&self) -> String {
        self.clone().host
    }
}

pub fn get_configuration() -> Configuration {
    load_configuration("./mockit.json".to_string())
}

pub fn get_route<'a>(
    routes: &'a Vec<Route>,
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
            host: String::from("http://localhost"),
        }
    })
}

#[cfg(test)]
#[path = "./configuration_test.rs"]
mod tests;
