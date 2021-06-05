use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub resource: String,
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    routes: Vec<Route>,
}

pub fn get_routes() -> Vec<Route> {
    load_configuration("./mockit.json").routes
}

pub fn get_route<'a>(routes: &'a Vec<Route>, path: &'a str) -> Option<&'a Route> {
    routes
        .iter()
        .filter(|i| path.ends_with(&i.path))
        .last()
}

fn load_configuration(loaction: &str) -> Configuration {
    let data = fs::read_to_string(&loaction).expect("Unable to read data");
    serde_json::from_str(&data).expect("Unable to read data")
}
