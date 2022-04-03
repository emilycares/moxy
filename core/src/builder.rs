use std::collections::HashMap;

use hyper::body::HttpBody;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body, Client, Method, Request, Response, Uri};
use hyper_tls::HttpsConnector;

//pub async fn build(request: Request<Body>) -> Option<String> {
//let response = fetch_request(request).await;

//match response {
//Some(response) => {
//let data = get_body(response).await;
//}
//None => todo!(),
//}

//None
//}

pub async fn fetch_request(request: Request<Body>, from: String) -> Option<Body> {
    let headers: HashMap<String, String> = HashMap::new();

    // fetch data
    let response = fetch_url(
        request.method().to_owned(),
        &get_url(&request.uri(), from),
        get_body(request.into_body()).await,
        headers,
        )
        .await?;

    let body = response.into_body();

    Some(body)
}

const MAX_ALLOWED_RESPONSE_SIZE: u64 = 1024;
fn is_body_to_lage(response: &Option<Response<Body>>) -> bool {

    match response {
        Some(response) => {
            let response_content_length = match &response.body().size_hint().upper() {
                Some(v) => v,
                None => &MAX_ALLOWED_RESPONSE_SIZE. + 1, // Just to protect ourselves from a malicious response
            };

            response_content_length > &MAX_ALLOWED_RESPONSE_SIZE
        },
        None => false,
    }
}

async fn get_body(body: Body) -> Option<String> {
    // size check

    // return result
    match hyper::body::to_bytes(body).await {
        Ok(bytes) => match String::from_utf8(bytes.to_vec()) {
            Ok(content) => Some(content),
            Err(err) => {
                println!("Unable to parse body content as utf8, {:?}", err);
                None
            }
        },
        Err(err) => {
            println!("Unable to get bytes form request, {:?}", err);
            None
        }
    }
}

fn get_url(uri: &Uri, host: String) -> String {
    host + &uri.to_string()
}

pub async fn fetch_url(
    method: Method,
    url: &str,
    body: Option<String>,
    header: HashMap<String, String>,
    ) -> Option<Response<Body>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    println!(
        "fetching: url: {}, header: {:?}, body: {:?}",
        &url, &header, &body
        );

    let response = match get_request(method, url, body, header) {
        Some(request) => match client.request(request).await {
            Ok(response) => Some(response),
            Err(err) => {
                println!("Server did not respond as expected {:?}", err);
                None
            }
        },
        None => {
            println!("Unable to perform request");
            None
        }
    }

    match is_body_to_lage(&response) {
        true => None,
        false => response,
    }
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
        Ok(r) => Some(r),
        Err(err) => {
            println!("Unable to build request {:?}", err);
            None
        }
    }
}

#[cfg(test)]
#[path = "./builder_test.rs"]
mod tests;
