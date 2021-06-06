use cached::proc_macro::cached;
use crate::configuration::{self, Route};
use std::fs;
use tide::{http::mime, Request, Response, Server};

pub fn setup(routes: Vec<Route>, server: &mut Server<()>) {
    for i in routes.iter() {
        if i.method.eq("get") {
            server.at(&i.path).get(endpoint);
        } else if i.method.eq("head") {
            server.at(&i.path).head(endpoint);
        } else if i.method.eq("post") {
            server.at(&i.path).post(endpoint);
        } else if i.method.eq("put") {
            server.at(&i.path).put(endpoint);
        } else if i.method.eq("delete") {
            server.at(&i.path).delete(endpoint);
        } else if i.method.eq("connect") {
            server.at(&i.path).connect(endpoint);
        } else if i.method.eq("trace") {
            server.at(&i.path).trace(endpoint);
        } else if i.method.eq("patch") {
            server.at(&i.path).patch(endpoint);
        }
    }
}

async fn endpoint(req: Request<()>) -> tide::Result<tide::Response> {
    let routes = configuration::get_routes();
    let route: &Route = &configuration::get_route(&routes, &req.url().as_str()).unwrap();

    let data = get_data(route.resource.to_string());
    let response = Response::builder(200)
        .content_type(get_mime(&data))
        .body(data)
        .build();
    Ok(response)
}


#[cached]
fn get_data(resource: String) -> String {
    println!("Load resource: {}", resource);
    fs::read_to_string(resource).unwrap_or_else(|_error| "File not found".to_string())
}

fn get_mime(path: &str) -> mime::Mime {
    if path.ends_with("json") {
        return mime::JSON;
    }

    mime::PLAIN
}
