//! Load routes from filesystem
use crate::configuration::Route;
use tokio::fs;

/// Call file with replaced parameter when there is a parameter.
pub async fn load(route: &Route, parameter: Option<&str>) -> Option<Vec<u8>> {
    match if let Some(parameter) = parameter {
        let dynamic_resource = route.resource.replace("$$$", parameter);
        file(dynamic_resource).await
    } else {
        file(route.resource.to_string()).await
    } {
        Ok(data) => Some(data.to_vec()),
        Err(_) => None,
    }
}

/// Load file for route.
async fn file(resource: String) -> Result<Vec<u8>, std::io::Error> {
    log::debug!("Load File: {}", resource);
    match fs::read(&resource).await {
        Ok(data) => Ok(data),
        Err(e) => Err(e),
    }
}
