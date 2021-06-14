use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::ErrorKind;

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
    if path.to_lowercase().contains(&"$$$".to_string()) {
        let (start, end) = get_not_dynamic_part_of_url(&path);
        routes
            .iter()
            .filter(|i| i.path.starts_with(start))
            .filter(|i| i.path.ends_with(end))
            .last()
    } else {
        routes.iter().filter(|i| path.ends_with(&i.path)).last()
    }
}

fn get_not_dynamic_part_of_url(path: &str) -> (&str, &str) {
    let wildcard = "$$$";
    (
        path.trim_end_matches(wildcard),
        path.trim_start_matches("$$$"),
    )
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
