use std::env;
use teloxide::{net::Download, requests::Requester, types::Document, Bot};

pub async fn add_file(bot: &Bot, user_id: String, document: &Document) -> String {
    let file_name = document.clone().file_name.unwrap();
    let user_path = get_user_path(user_id.to_owned());
    let file_path = get_file_path(user_id.to_owned(), file_name);

    // create user directory if it's missing
    if !tokio::fs::try_exists(&user_path).await.unwrap() {
        tokio::fs::create_dir(&user_path).await.unwrap();
    }

    // check if user has already 3 files
    match std::fs::read_dir(&user_path) {
        Ok(result) => {
            if result.count() >= 3 {
                return "already 3 files stored".to_owned();
            }

            // check if file already exists
            match tokio::fs::try_exists(&file_path).await {
                Ok(exists) => {
                    if exists {
                        // bot.send_message(msg.chat.id, "file with the same name was already saved")
                        //     .await?;
                        return "file with the same name was already stored".to_owned();
                    }
                }
                Err(_err) => {
                    return "could not test if file already exists".to_owned();
                }
            }

            let mut dst = tokio::fs::File::create(&file_path).await.unwrap();
            // let writer = BufWriter::new(dst);

            let file = bot.get_file(&document.file.id).await.unwrap();
            // bot.download_file(&file.path, &mut dst).await.unwrap();
            bot.download_file(&file.path, &mut dst).await.unwrap();
            return "file added".to_owned();
        }
        Err(_err) => {
            return "could not check directory".to_owned();
        }
    }
}

pub async fn remove_file(user_id: String, file_name: String) -> bool {
    let file_path = get_file_path(user_id, file_name);
    if !tokio::fs::try_exists(&file_path).await.unwrap() {
        return false;
    } else {
        tokio::fs::remove_file(file_path).await.unwrap();
        return true;
    }
}

pub async fn list_files(user_id: String) -> Vec<String> {
    let user_path = get_user_path(user_id);
    let mut result: Vec<String> = Vec::new();
    if tokio::fs::try_exists(&user_path).await.unwrap() {
        let mut read_dir = tokio::fs::read_dir(&user_path).await.unwrap();

        loop {
            let mut stop = false;
            match read_dir.next_entry().await.unwrap() {
                Some(e) => result.push(e.file_name().to_str().unwrap().to_string()),
                None => {
                    stop = true;
                }
            }
            if stop {
                break;
            }
        }
    }
    return result;
}

pub async fn get_file_for_user(user_id: String, file_name: String) -> Option<String> {
    let file_path = get_file_path(user_id, file_name);
    if !tokio::fs::try_exists(&file_path).await.unwrap() {
        return None;
    }
    return Some(file_path);
}

pub fn get_base_path() -> String {
    return env::var("GPX_TRACK_BOT_DATA").unwrap().to_string();
}

fn get_user_path(user_id: String) -> String {
    return format!("{}/{}", get_base_path(), user_id);
}

fn get_file_path(user_id: String, file_name: String) -> String {
    return format!("{}/{}", get_user_path(user_id), file_name);
}
