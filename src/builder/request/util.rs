use std::{collections::HashMap, str::FromStr};

use hyper::{
    header::{HeaderName, HeaderValue},
    HeaderMap, Uri,
};

/// Get url with the new host
pub fn get_url(uri: &Uri, host: impl Into<String>) -> String {
    host.into() + &uri.to_string()
}

/// Get url with the new host
pub fn get_url_str(uri: &str, host: impl Into<String>) -> String {
    host.into() + uri
}

/// Convert HashMap to HeaderMap
pub fn hash_map_to_header_map(map: HashMap<String, String>) -> HeaderMap {
    let keys = map.keys();
    let mut out = HeaderMap::new();

    for key in keys {
        if let Some(value) = map.get(key) {
            let key = HeaderName::from_str(key.to_owned().as_str());
            if let Ok(key) = key {
                let value = HeaderValue::from_str(value.to_owned().as_str());
                if let Ok(value) = value {
                    out.insert(key, value);
                }
            }
        }
    }

    out
}

/// Convert Headermap to HashMap
pub fn header_map_to_hash_map(header: &HeaderMap) -> HashMap<String, String> {
    let keys = header.keys();
    let mut out: HashMap<String, String> = HashMap::new();

    for key in keys {
        if let Some(value) = header.get(key) {
            if let Ok(value) = value.to_owned().to_str() {
                out.insert(key.to_string(), value.to_string());
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{header_map_to_hash_map, hash_map_to_header_map};


    #[test]
    fn validate_hashmap_conversion() {
        let mut hash_map: HashMap<String, String> = HashMap::new();

        hash_map.insert(String::from("a"), String::from("b"));
        hash_map.insert(String::from("b"), String::from("c"));
        hash_map.insert(String::from("c"), String::from("d"));

        assert_eq!(header_map_to_hash_map(&hash_map_to_header_map(hash_map.clone())), hash_map)
    }
}
