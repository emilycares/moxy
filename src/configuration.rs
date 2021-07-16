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
    let clear = routes.iter().filter(|i| path.ends_with(&i.path)).last();

    if clear.is_some() {
        clear
    } else {
        let dynamic_routes = routes
            .iter()
            .filter(|i| i.path.contains("$$$"))
            .filter(|i| {
                let index = i.path.find("$$$").unwrap();
                let port_end = path.find("8080").unwrap() + 4;
                let match_before = &i.path[0..index];

                let checkable_path = &path[port_end..path.len()];

                checkable_path.starts_with(&match_before)
                // TODO: check end
                //let start = &index + 3;
                //let match_after = &i.path[start..i.path.len() - 1];
                //println!("{}", match_after);

                //path.ends_with(match_after)
            });

        dynamic_routes.last()
    }

    //if path.to_lowercase().contains(&"$$$".to_string()) {
    //let (start, end) = get_start_and_end(&path);
    //routes
    //.iter()
    //.filter(|i| i.path.starts_with(start))
    //.filter(|i| i.path.ends_with(end))
    //.last()
    //}
}

//fn get_start_and_end(path: &str) -> (&str, &str) {
//let wildcard = "$$$";
//(
//path.trim_end_matches(wildcard),
//path.trim_start_matches(wildcard),
//)
//}

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
