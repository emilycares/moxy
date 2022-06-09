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
    if config.get_route(&path, method.clone()).is_none() {
        let route = Route {
            method: method.clone(),
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

        match check_existing_file(folders.as_str()).await {
            Ok(path_changes) => {
                for (from, to) in path_changes {
                    if let Some(route) = config.get_route_mut(&from.to_owned(), method.clone()) {}
                }
            }
            Err(e) => return Err(e),
        }
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
async fn check_existing_file(folders: &str) -> Result<Vec<(String, String)>, std::io::Error> {
    // Remove file if there was one
    log::trace!("{:?}", folders);

    let path_changes = vec![];

    for f in get_folders_to_check(folders) {
        match folder_check(f).await {
            Ok(c) => path_changes.push((f, c)),
            Err(e) => return Err(e),
        }
    }

    Ok(path_changes)
}

async fn folder_check(folder: String) -> Result<Option<String>, std::io::Error> {
    if Path::new(&folder).is_file() {
        let prefious_file = Some(fs::read(&folder).await);
        fs::remove_file(&folder).await?;
        fs::create_dir_all(&folder).await?;
        let path = folder.to_owned() + "/index";
        let mut index_file = File::create(&path).await?;
        if let Some(Ok(prefious_file)) = prefious_file {
            index_file.write_all(&prefious_file).await?
        }

        return Ok(Some(path));
    }

    Ok(None)
}

fn get_folders_to_check(folders: &str) -> Vec<String> {
    let lft: Vec<&str> = folders.split('/').collect();

    let length = lft.len() + 1;
    let mut checks = vec![];

    for i in 1..length {
        let mut check = String::from("");
        for y in 0..i {
            check += &lft[y];
            if y + 1 != i {
                check += "/";
            }
        }

        checks.push(check);
    }

    checks
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

mod test {
    use crate::builder::storage::get_folders_to_check;

    #[test]
    fn get_folders_to_check_should_return_correct_result() {
        let input = "./db/api/asdf-service/user/micmine";

        let expected = vec![
            ".",
            "./db",
            "./db/api",
            "./db/api/asdf-service",
            "./db/api/asdf-service/user",
            "./db/api/asdf-service/user/micmine",
        ];

        assert_eq!(get_folders_to_check(input), expected);
    }
}
