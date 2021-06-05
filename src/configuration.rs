use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
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
    routes
        .iter()
        .filter(|i| path.ends_with(&i.path))
        .last()
}

#[cached]
fn load_configuration(loaction: String) -> Configuration {
    println!("Load Configuration: {}", loaction);
    let data = fs::read_to_string(&loaction).expect("Unable to read data");
    serde_json::from_str(&data).expect("Unable to read data")
}
