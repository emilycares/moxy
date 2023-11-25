/// Get url with the new host
pub fn get_url(uri: &str, host: impl Into<String>) -> String {
    host.into() + uri
}

/// Get url with the new host
pub fn get_url_str(uri: &str, host: impl Into<String>) -> String {
    host.into() + uri
}
