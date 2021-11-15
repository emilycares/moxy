use std::collections::HashMap;

//type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

//pub fn build(original: &Request<Body>, proxy: &str) {}

pub fn fetch_url(url: &str, header: HashMap<String, String>) -> Result<String, ureq::Error> {
    let _body: ureq::Request = ureq::get(&url);

    for (name, value) in header.iter() {
        _body.set(name, value);
    }

    let result = _body.call()?.into_string()?;

    Ok(result)
}

//fn get_proxy_uri(uri: &Uri, proxy: &str) -> Uri {
//Uri::builder()
//.authority(proxy)
//.scheme(uri.scheme().unwrap().as_str())
//.path_and_query(uri.path_and_query().unwrap().as_str())
//.build()
//.unwrap()
//}
