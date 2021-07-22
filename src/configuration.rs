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
}

pub fn get_routes() -> Vec<Route> {
    load_configuration("./mockit.json".to_string()).routes
}

pub fn get_route<'a>(
    routes: &'a Vec<Route>,
    uri: &'a Uri,
    method: &'a str,
) -> (Option<&'a Route>, Option<&'a str>) {
    for i in routes.iter() {
        println!("{}, {}", i.method, &method);
        if i.method.eq(&method) {
            let idx = &i.path.find("$$$");
            let path = &uri.path();

            if idx.is_some() {
                let index = &idx.unwrap();
                let match_before = &i.path[0..*index];
                println!("match_before: {}", match_before);
                println!("checkable_path: {}", path);

                if path.starts_with(&match_before) {
                    //println!("start maches");
                    if index + 3 != i.path.len() {
                        let match_end = &i.path[index + 3..i.path.len()];

                        println!("match_end: {}", match_end);
                        if path.ends_with(match_end) {
                            let sd = match_end.len();
                            return (Some(i), Some(&path[i.path.len() - 3 - sd..path.len() - sd]));
                        }
                    } else {
                        println!("No more after dyn, {}", &path);
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
