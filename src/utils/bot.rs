use crate::utils::files::add_file;
use std::env;

use crate::utils::commands::Command;
use crate::utils::files::get_file_for_user;
use crate::utils::gpx::generate_partial_gpx;

use serde_json::Value;
use teloxide::types::InputFile;
use teloxide::{
    dispatching::DpHandlerDescription, prelude::*, types::ParseMode, utils::command::BotCommands,
    RequestError,
};

const INFO_TEXT: &str = r"*General Information* 

\- to use this bot you have to be whitelisted by the hoster of the bot

\- interactions with this bot can either be a command of the supported commands or a message containing a gpx file

\- if you send a `\*\.gpx` file to the bot it will be stored and made available only to you

\- you can store a maximum of 3 files";

pub fn build_dp_tree(
) -> Handler<'static, DependencyMap, Result<(), RequestError>, DpHandlerDescription> {
    return dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .filter_map(|cmd: Command| match cmd {
                    // Command::Help | Command::Start | Command::RequestAccess => Some(()),
                    Command::Help | Command::Start => Some(()),
                    _ => None,
                })
                .endpoint(answer),
        )
        .branch(
            Update::filter_message()
                .filter_map(|msg: Message| {
                    let env_allowed_users: Vec<UserId> = env::var("GPX_TRACK_BOT_ALLOWED_USERS")
                        .unwrap_or("".to_string())
                        .split(",")
                        .map(|id| teloxide::prelude::UserId(id.parse().unwrap()))
                        .collect();

                    if env_allowed_users.contains(&msg.from().unwrap().id) {
                        None
                    } else {
                        Some(())
                    }
                })
                .endpoint(permission_denied),
        )
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(answer),
        )
        .branch(Update::filter_message().endpoint(default_message_handler));
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help | Command::Start => {
            bot.clone()
                .parse_mode(ParseMode::MarkdownV2)
                .send_message(msg.chat.id, INFO_TEXT)
                .await?;
            let text = String::from("Supported Commands: \n\n")
                + Command::bot_commands()
                    .to_vec()
                    .iter()
                    .map(|x| {
                        if x.description.to_string().len() > 0 {
                            format!(r"{} — {}", x.command, x.description.to_string())
                        } else {
                            format!("{}", x.command)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
                    .as_str();
            bot.clone()
                .parse_mode(ParseMode::MarkdownV2)
                .send_message(msg.chat.id, text)
                .await?;
        }
        Command::AddressCut {
            filename,
            distance,
            start_address,
        } => {
            bot.send_message(msg.chat.id, "preparing gpx please wait")
                .await?;
            let url = format!(
                "https://nominatim.openstreetmap.org/search?q={}&format=json",
                start_address
            );
            let string_response = reqwest::Client::new()
                .get(url)
                .header("User-Agent", "TelegramBot")
                .send()
                .await?
                .text()
                .await?;
            let v: Value = serde_json::from_str(&string_response).unwrap();
            let lon: f64 = v[0]["lon"].as_str().unwrap().parse().unwrap();
            let lat: f64 = v[0]["lat"].as_str().unwrap().parse().unwrap();
            handle_gpx(bot, msg, filename, distance, lon, lat).await;
        }
        Command::CoordinatesCut {
            filename,
            distance,
            longitude,
            latitude,
        } => {
            bot.send_message(msg.chat.id, "preparing gpx please wait")
                .await?;
            println!("longitued: {}", longitude.parse::<f64>().unwrap());
            println!("latitude: {}", latitude.parse::<f64>().unwrap());
            handle_gpx(
                bot,
                msg,
                filename,
                distance,
                longitude.parse::<f64>().unwrap(),
                latitude.parse::<f64>().unwrap(),
            )
            .await;
        }

        Command::ListFiles => {
            let files = super::files::list_files(msg.from().unwrap().id.to_string()).await;
            if files.len() > 0 {
                bot.send_message(
                    msg.chat.id,
                    String::from("stored files:\n")
                        + &files.iter().map(|x| format!("- {}", x)).collect::<String>(),
                )
                .await?;
            } else {
                bot.send_message(msg.chat.id, "you don't have any files stored")
                    .await?;
            }
        }

        Command::DeleteFile { filename } => {
            let ok = super::files::remove_file(msg.from().unwrap().id.to_string(), filename).await;
            if ok {
                bot.send_message(msg.chat.id, "sucessfully removed file")
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "cold not remove file")
                    .await?;
            }
        }
    };
    Ok(())
}

async fn handle_gpx(
    bot: Bot,
    msg: Message,
    filename: String,
    expected_distance: u32,
    longitude: f64,
    latitude: f64,
) {
    let awaited_result = get_file_for_user(msg.from().unwrap().id.to_string(), filename).await;

    if awaited_result.is_none() {
        let _ = bot
            .send_message(msg.chat.id, "could not resolve file")
            .await;
        return;
    }
    let file_path = awaited_result.unwrap();

    match generate_partial_gpx(
        file_path.to_string(),
        expected_distance,
        longitude,
        latitude,
    ) {
        Some(filepath) => {
            bot.send_document(msg.chat.id, InputFile::file(&filepath))
                .await
                .unwrap();
            std::fs::remove_file(filepath).unwrap();
        }
        None => {
            let _ = message_handler(bot, msg).await;
        }
    }
}

async fn permission_denied(bot: Bot, msg: Message) -> ResponseResult<()> {
    println!("message id {}", msg.from().unwrap().id);
    bot.send_message(msg.chat.id, "you don't have permission to use this bot; create a issue on https://github.com/bb4L/gpx-track-bot if you want to use the bot")
    .await?;
    Ok(())
}

async fn default_message_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    match msg.document() {
        Some(document) => {
            match &document.mime_type {
                Some(mime_type) => {
                    println!("file type {}", mime_type);
                    if mime_type.to_string() == "application/gpx+xml" {
                        bot.send_message(msg.chat.id, "handling file").await?;
                        let response =
                            add_file(&bot.clone(), msg.from().unwrap().id.to_string(), document)
                                .await;
                        bot.send_message(msg.chat.id, response).await?;
                    } else {
                        bot.send_message(msg.chat.id, "wrong type").await?;
                    }
                }
                None => {
                    println!("unclear mime type");
                }
            }
            Ok(())
        }
        None => message_handler(bot, msg).await,
    }
}
async fn message_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    match msg.text() {
        None => {
            bot.send_message(msg.chat.id, format!("could not handle {}", "message"))
                .await?;
        }
        Some(text) => {
            bot.send_message(msg.chat.id, format!("could not handle message '{}'", text))
                .await?;
        }
    }
    Ok(())
}
