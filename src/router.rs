use crate::{
    configuration::{self, Route},
    data_loader,
};
use tide::{http::mime, Request, Response, Server};

pub fn setup(routes: Vec<Route>, server: &mut Server<()>) {
    for i in routes.iter() {
        let static_path = &i.path.replace("$$$", ":parameter");
        if i.method.eq("get") {
            server.at(&static_path).get(endpoint);
        } else if i.method.eq("head") {
            server.at(&static_path).head(endpoint);
        } else if i.method.eq("post") {
            server.at(&static_path).post(endpoint);
        } else if i.method.eq("put") {
            server.at(&static_path).put(endpoint);
        } else if i.method.eq("delete") {
            server.at(&static_path).delete(endpoint);
        } else if i.method.eq("connect") {
            server.at(&static_path).connect(endpoint);
        } else if i.method.eq("trace") {
            server.at(&static_path).trace(endpoint);
        } else if i.method.eq("patch") {
            server.at(&static_path).patch(endpoint);
        }
    }
}

async fn endpoint(req: Request<()>) -> tide::Result<tide::Response> {
    println!("{}", req.url().as_str());
    let routes = configuration::get_routes();
    let route = configuration::get_route(&routes, &req.url().as_str());

    if route.is_some() {
        let data = data_loader::load(&route.unwrap(), req.param("parameter"));
        let response = Response::builder(200)
            .content_type(get_mime(&data))
            .body(data)
            .build();
        Ok(response)
    } else {
        let response = Response::builder(404)
            .content_type(mime::PLAIN)
            .body(format!(
                "Could not load resource for path {}",
                &req.url().as_str()
            ))
            .build();
        Ok(response)
    }
}

fn get_mime(path: &str) -> mime::Mime {
    if path.ends_with("json") {
        return mime::JSON;
    }

    mime::PLAIN
}
