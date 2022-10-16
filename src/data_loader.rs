//! Load routes from filesystem
use crate::configuration::Route;

/// Call file with replaced parameter when there is a parameter.
pub async fn load(route: &Route, parameter: Option<&str>) -> Option<Vec<u8>> {
    if let Some(resource) = &route.resource {
        return match if let Some(parameter) = parameter {
            let dynamic_resource = resource.replace("$$$", parameter);
            file(dynamic_resource.as_str()).await
        } else {
            file(resource).await
        } {
            Ok(data) => Some(data.to_vec()),
            Err(_) => None,
        };
    }

    None
}

/// Load file for route.
pub async fn file(resource: &str) -> Result<Vec<u8>, std::io::Error> {
    tracing::trace!("Load File: {}", resource);
    match tokio::fs::read(&resource).await {
        Ok(data) => Ok(data),
        Err(e) => Err(e),
    }
}

/// Load file for route.
pub fn file_sync(resource: &str) -> Result<Vec<u8>, std::io::Error> {
    tracing::trace!("Load File: {}", resource);
    std::fs::read(&resource)
}
