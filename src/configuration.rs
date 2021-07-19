use cached::proc_macro::cached;
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
}

pub fn get_routes() -> Vec<Route> {
    load_configuration("./mockit.json".to_string()).routes
}

pub fn get_route<'a>(routes: &'a Vec<Route>, path: &'a str) -> Option<&'a Route> {
    for i in routes.iter() {
        let idx = &i.path.find("$$$");

        if idx.is_some() {
            let index = &idx.unwrap();
            let port_end = path.find("8080").unwrap() + 4;
            let match_before = &i.path[0..*index];

            let checkable_path = &path[port_end..path.len()];

            if checkable_path.starts_with(&match_before) {
                if index + 3 != i.path.len() {
                    let end = &i.path[index + 3..i.path.len()];

                    if path.ends_with(end) {
                        return Some(i);
                    }
                }
            }
        }
        if path.ends_with(&i.path) {
            return Some(i);
        }
    }

    None
}

#[cached]
fn load_configuration(loaction: String) -> Configuration {
    println!("Load Configuration: {}", loaction);
    let data = fs::read_to_string(&loaction).unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            File::create(loaction)
                .unwrap_or_else(|error| panic!("Could not open configuration file: {:?}", error));
        } else {
            println!("Could not open configuration file: {:?}", error);
        }
        "".to_string()
    });

    serde_json::from_str(&data).unwrap_or_else(|error| {
        println!("Could not load configuration file: {:?}", error);
        Configuration { routes: vec![] }
    })
}

#[cfg(test)]
#[path = "./configuration_test.rs"]
mod tests;
