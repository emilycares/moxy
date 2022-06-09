use std::collections::HashMap;

use hyper::Uri;

use crate::{
    builder::{request, storage::get_save_path},
    configuration::{RouteMethod, Route, get_route},
};

#[tokio::test]
async fn requrest_no_body() {
    let _reponse = request::fetch_request(
        RouteMethod::GET,
        "http://example.com".to_string(),
        None,
        HashMap::new(),
    )
    .await
    .unwrap();
}

#[test]
fn get_save_path_should_start_with_db() {
    let path = get_save_path("/index.html");

    assert!(&path.starts_with("./db"));
}

#[test]
fn get_save_path_should_add_index_if_folder() {
    let path = get_save_path("/");

    assert!(&path.ends_with("/index"));
}

#[test]
fn get_route_should_not_find_entry_if_the_url_only_partialy_matches() {
    let routes = [Route {
        method: RouteMethod::GET,
        path: "/a".to_string(),
        resource: "".to_string()
    }];

    let uri = Uri::from_static("/a/test");

    let result = get_route(&routes, &uri, RouteMethod::GET);

    assert_eq!(result, (None, None));
}
