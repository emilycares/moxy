use std::{path::Path, sync::Arc};

use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    sync::Mutex,
};

use crate::configuration::{self, Configuration, Route, RouteMethod};

/// Modifies the configuration and filesystem to add more entryes
pub async fn save(
    method: RouteMethod,
    uri: &str,
    body: Vec<u8>,
    config: Arc<Mutex<Configuration>>,
) -> Result<(), std::io::Error> {
    let path = get_save_path(uri);
    let mut config = config.lock().await;
    if !config.has_route(&path, method.clone()) {
        let route = Route {
            method,
            resource: path.clone(),
            path: uri.to_owned(),
        };
        log::info!("Save route: {:?}", route);

        config.routes.push(route);

        let folders: String = if let Some(index) = path.rfind('/') {
            path[0..index].to_owned()
        } else {
            path.to_owned()
        };

        check_existing_file(path.as_str(), folders.as_str()).await?;
        save_file(path.as_str(), body).await?;
        configuration::save_configuration(config.to_owned()).await?;
    }

    Ok(())
}


/// This function will check if there is a file in the current folder structure.
/// Previous: Triggered with a call to /api/some-service/results
/// folders:
///   db/
///     api/
///       some-service/
///         results (file)
/// 
/// Next: Triggered with a call to /api/some-service/results/micmine
/// Wanted folder structure: 
///   db/
///     api/
///       some-service/
///         results/
///           index (file, prefious file "db/api/some-service/results")
///           micmine (file)
///
/// In order to create the file micmine we need to create the folders and need to move awaiy
/// any existing files that colide with the folder path.
async fn check_existing_file(location: &str, folders: &str) -> Result<(), std::io::Error> {
    // Remove file if there was one
    if Path::new(&folders).is_file() {
        let prefious_file = Some(fs::read(&location).await);
        fs::remove_file(&location).await?;
        fs::create_dir_all(&folders).await?;
        let mut index_file = File::create(folders.to_owned() + "/index").await?;
        if let Some(Ok(prefious_file)) = prefious_file {
            index_file.write_all(&prefious_file).await?
        }
    }
    Ok(())
}

/// Saves a file to the expected location
async fn save_file(location: &str, body: Vec<u8>) -> Result<(), std::io::Error> {
    let mut file = File::create(location).await?;
    file.write_all(&body).await?;

    Ok(())
}

/// Will generate a file location based on a uri.
pub fn get_save_path(uri: &str) -> String {
    let mut path = "./db".to_owned() + uri;

    if path.ends_with('/') {
        path += "index"
    }

    path
}
