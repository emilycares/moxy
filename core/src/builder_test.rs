use std::collections::HashMap;

use hyper::{Method, Body};

use super::{get_body, fetch_request};

#[tokio::test]
async fn requrest_no_body() {
    let reponse = fetch_request(
        Method::GET,
        "http://google.com".to_string(),
        None,
        HashMap::new(),
        )
        .await.unwrap();
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
