use std::fs;

use cached::proc_macro::cached;
use tide::Request;

use crate::configuration::Route;

pub fn load(route: &Route, current: &Request<()>) -> String {
    //if route.resource.starts_with("htt") {
        "unimplemented".to_string()
    //} else {
        //if route.path.contains("$$$") {
            //let url = current.url().as_str();
        //} else {
            //file(route.resource.as_str)
        //}
    //}
}

//fn get_wildcard_data<'a>(route_path: &'a str, url: &'a str) -> &str {
    //let start = route_path.find("$$$");
    
//}
//#[test]
//fn test1_get_wildcard_data() {
    //assert_eq!(get_wildcard_data("/api/test/$$$.json", "/api/test/asdf.json"), "asdf");
//}

#[cached]
fn file(resource: String) -> String {
    println!("Load File: {}", resource);
    fs::read_to_string(resource).unwrap_or_else(|_error| "File not found".to_string())
}
