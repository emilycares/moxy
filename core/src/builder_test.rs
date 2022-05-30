use std::collections::HashMap;

use hyper::{Body, Method};

use crate::builder::{get_body, get_save_path};

use super::fetch_request;

#[tokio::test]
async fn requrest_no_body() {
    let reponse = fetch_request(
        Method::GET,
        "http://google.com".to_string(),
        None,
        HashMap::new(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn get_body_should_return_some_of_empty_string_when_body_is_empty() {
    let body = get_body(Body::empty()).await;

    assert_eq!(body, Some(String::from("")))
}

#[tokio::test]
async fn get_body_should_return_none_when_body_is_to_large() {
    let body = get_body(Body::empty()).await;
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
