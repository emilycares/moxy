use std::collections::HashMap;

use crate::{
    builder::{request, storage::get_save_path},
    configuration::RouteMethod,
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
