use crate::configuration::Route;
use cached::proc_macro::cached;
use std::fs;

pub fn load(route: &Route, parameter: Option<&str>) -> String {
    if route.resource.starts_with("htt") {
        "unimplemented".to_string()
    } else if let Some(parameter) = parameter {
        let dynamic_resource = route.resource.replace("$$$", parameter);
        file(dynamic_resource)
    } else {
        file(route.resource.to_string())
    }
}

#[cached]
fn file(resource: String) -> String {
    log::debug!("Load File: {}", resource);
    fs::read_to_string(&resource).unwrap_or_else(|_| format!("File: {} not found", resource))
}
