use std::collections::HashMap;

use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body, Client, Method, Request, Response};

async fn fetch_url(
    method: Method,
    url: &str,
    body: Option<String>,
    header: HashMap<String, String>,
) -> Option<Response<Body>> {
    let client = Client::new();

    if let Some(request) = get_request(method, url, body, header) {
        match client.request(request).await {
            Ok(response) => Some(response),
            Err(_) => None
        }
    } else {
        None
    }
}

#[tokio::test]
async fn requrest_no_body() {
    let reponse = fetch_url(
        Method::GET,
        "http://google.com",
        None,
        HashMap::new(),
    )
    .await.unwrap();
}

fn get_request(
    method: Method,
    url: &str,
    body: Option<String>,
    header: HashMap<String, String>,
) -> Option<Request<Body>> {
    let mut req = Request::builder().uri(url).method(&method); // create request
    if let Some(request_header) = req.headers_mut() {
        // get mutable header
        for (name, value) in header {
            request_header.insert(
                HeaderName::from_lowercase(name.as_bytes()).unwrap(),
                HeaderValue::from_str(&value).unwrap_or(HeaderValue::from_static("")),
            );
        }
    }

    let body = match body {
        Some(data) => Body::from(data),
        None => Body::empty(),
    };

    let request = req.body(body);

    match request {
        Ok(r) => {
            Some(r)
        }
        Err(_) => None,
    }
}

//fn get_proxy_uri(uri: &Uri, proxy: &str) -> Uri {
//Uri::builder()
//.authority(proxy)
//.scheme(uri.scheme().unwrap().as_str())
//.path_and_query(uri.path_and_query().unwrap().as_str())
//.build()
//.unwrap()
//}
