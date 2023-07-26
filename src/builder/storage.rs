use futures_util::future;
use std::{path::Path, sync::Arc};

use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    sync::Mutex,
};

use crate::configuration::{self, Configuration, Route, RouteMethod, WsMessage, WsMessageType};

use super::ws::WsClientMessage;

/// Modifies the configuration and filesystem to add more entryes
pub async fn save(
    method: &RouteMethod,
    uri: &str,
    metadata: Option<configuration::Metadata>,
    body: Vec<u8>,
    config: Arc<Mutex<Configuration>>,
) -> Result<(), std::io::Error> {
    let content_type: Option<String> = metadata
        .clone()
        .unwrap_or_default()
        .header
        .get("content-type")
        .cloned()
        .map(|v| v.to_str().unwrap_or_default().to_string());
    let path = get_save_path(uri, content_type.as_deref());
    let mut config = config.lock().await;
    if config.get_route(&path, method).is_none() {
        let route = Route {
            method: method.clone(),
            metadata,
            resource: Some(path.clone()),
            path: uri.to_owned(),
            messages: vec![],
        };
        tracing::info!("Save route: {:?}", route);

        config.routes.push(route);

        let folders = get_folders(&path);

        match check_existing_file(folders.as_str()).await {
            Ok(resource_changes) => {
                for (from, to) in resource_changes {
                    if let Some(route) = config.get_route_by_resource_mut(&from.to_owned(), method)
                    {
                        route.resource = Some(to.to_string());
                    }
                }
            }
            Err(e) => return Err(e),
        }
        save_file(path.as_str(), body, folders.as_str()).await?;
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
    let mut path_changes = vec![];

    for f in get_folders_to_check(folders) {
        match folder_check(&f).await {
            Ok(c) => {
                if let Some(c) = c {
                    path_changes.push((f, c));
                }
            }
            Err(e) => return Err(e),
        }
    }

    Ok(path_changes)
}

async fn folder_check(folder: &str) -> Result<Option<String>, std::io::Error> {
    if Path::new(&folder).is_file() {
        let prefious_file = Some(fs::read(&folder).await);
        fs::remove_file(&folder).await?;
        fs::create_dir_all(&folder).await?;
        let path = folder.to_owned() + "/index";
        let mut index_file = File::create(&path).await?;
        if let Some(Ok(prefious_file)) = prefious_file {
            index_file.write_all(&prefious_file).await?;
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
        for (y, _item) in lft.iter().enumerate().take(i) {
            check += lft[y];
            if y + 1 != i {
                check += "/";
            }
        }

        checks.push(check);
    }

    checks
}

/// Save websocket messages on the file system
pub async fn save_ws_client_message(path: &str, messages: Vec<WsClientMessage>) -> Vec<WsMessage> {
    tracing::trace!("save: {:?}", messages);
    let messages: Vec<(WsMessage, Vec<u8>)> = messages
        .iter()
        .enumerate()
        .map(|(i, message)| {
            let mut path = path.to_owned() + "_ws/" + &i.to_string();

            if is_json(&message.content) {
                path += ".json";
            }

            let path = get_save_path(path.as_str(), None);

            if message.offset <= 5 {
                (
                    WsMessage {
                        kind: WsMessageType::Startup,
                        time: None,
                        location: path,
                    },
                    message.content.clone(),
                )
            } else {
                let time = message.offset.to_string() + "s";
                (
                    WsMessage {
                        kind: WsMessageType::After,
                        time: Some(time),
                        location: path,
                    },
                    message.content.clone(),
                )
            }
        })
        .collect();

    future::try_join_all(
        messages
            .clone()
            .iter()
            .map(|(message, content)| async move {
                let folders = get_folders(&message.location);

                //return Ok(message.clone());

                match check_existing_file(folders.as_str()).await {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
                match save_file(&message.location, content.clone(), folders.as_str()).await {
                    Ok(_) => Ok(message.clone()),
                    Err(e) => Err(e),
                }
            }),
    )
    .await
    .unwrap();

    return messages.iter().map(|(msg, _)| msg.clone()).collect();
}

fn is_json(message: &[u8]) -> bool {
    match std::str::from_utf8(message) {
        Ok(message) => {
            let json: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(message);

            json.is_ok()
        }
        Err(_) => false,
    }
}

/// Saves a file to the expected location
async fn save_file(location: &str, body: Vec<u8>, folder: &str) -> Result<(), std::io::Error> {
    fs::create_dir_all(&folder).await?;
    let mut file = File::create(location).await?;
    file.write_all(&body).await?;

    Ok(())
}

const FALLBACK_CHAR: &str = "_";

/// Will generate a file location based on a uri.
pub fn get_save_path(uri: &str, content_type: Option<&str>) -> String {
    let uri = uri
        .replace('*', FALLBACK_CHAR)
        .replace('?', FALLBACK_CHAR)
        .replace('"', FALLBACK_CHAR)
        .replace('<', FALLBACK_CHAR)
        .replace('>', FALLBACK_CHAR)
        .replace(':', FALLBACK_CHAR)
        .replace('|', FALLBACK_CHAR);

    let file_suffix = if uri.ends_with(".txt") || uri.ends_with(".json") {
        Some(String::new())
    } else {
        get_extension(content_type)
    };
    let mut path = "./db".to_owned() + &uri.to_string();

    if path.ends_with('/') {
        path += "index";
    }

    if let Some(file_suffix) = file_suffix {
        if !path.ends_with(&file_suffix) {
            path += ".";
            path += &file_suffix;
        }
    } else {
        path += ".txt";
    }

    path
}

/// convert content_type to filetype
fn get_extension(content_type: Option<&str>) -> Option<String> {
    if let Some(content_type) = content_type {
        let content_type = if content_type.contains(';') {
            if let Some(content_type) = content_type.split(';').next() {
                content_type
            } else {
                &content_type
            }
        } else {
            &content_type
        };

        let mime = content_type.parse::<mime::Mime>();
        if mime.is_ok() {
            if let Some(mime) = mime_guess::get_mime_extensions_str(content_type) {
                return mime.first().map(|a| a.to_string());
            }
        }

        None
    } else {
        None
    }
}

/// convert filetype to content_type
pub fn get_content_type(file_name: String) -> String {
    let guess = mime_guess::from_path(file_name).first();

    if let Some(guess) = guess {
        return guess.to_string();
    }

    "text/plain".to_owned()
}

fn get_folders(path: &str) -> String {
    if let Some(index) = path.rfind('/') {
        path[0..index].to_owned()
    } else {
        path.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::storage::{get_folders_to_check, get_save_path, is_json};

    #[test]
    fn get_folders_to_check_should_return_correct_result_1() {
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

    #[test]
    fn get_folders_to_check_should_return_correct_result_2() {
        let input = "./db/a";

        let expected = vec![".", "./db", "./db/a"];

        assert_eq!(get_folders_to_check(input), expected);
    }

    #[test]
    fn get_save_path_add_db_folder() {
        let input = "/api/some-service/micmine";
        let expected = "./db/api/some-service/micmine.txt";

        assert_eq!(get_save_path(input, None), expected);
    }

    #[test]
    fn get_save_path_add_index_for_folder() {
        let input = "/api/some-service/micmine/";
        let expected = "./db/api/some-service/micmine/index.txt";

        assert_eq!(get_save_path(input, None), expected);
    }

    #[test]
    fn get_save_path_should_start_with_db() {
        let path = get_save_path("/index.html", None);

        assert!(&path.starts_with("./db"));
    }

    #[test]
    fn get_save_path_should_add_index_if_folder() {
        let path = get_save_path("/", None);

        assert!(&path.ends_with("/index.txt"));
    }

    #[test]
    fn is_json_should_detect_json() {
        let json_data = "{ \"some\": \"data\", \"and_a_number\": 1 }".as_bytes();

        assert!(is_json(json_data));
    }

    #[test]
    fn is_json_should_detect_invalid_json() {
        let json_data = "{ \"some\": \"data\"".as_bytes();

        assert!(!is_json(json_data));
    }

    #[test]
    fn is_json_should_detect_invalid_text() {
        let json_data: &[u8] = &[0, 1];

        assert!(!is_json(json_data));
    }
}
