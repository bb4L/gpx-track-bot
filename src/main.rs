pub mod utils;
use std::env;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting gpx-track-bot...");

    // check env
    if env::var("GPX_TRACK_BOT_ALLOWED_USERS").unwrap_or("no_allowed_users".to_string())
        == "no_allowed_users"
    {
        panic!("you have to set GPX_TRACK_BOT_ALLOWED_USERS")
    }

    if env::var("GPX_TRACK_BOT_DATA").unwrap_or("no_data_path".to_string()) == "no_data_path" {
        panic!("you have to set GPX_TRACK_BOT_DATA")
    }

    let base_path = utils::files::get_base_path();
    if !tokio::fs::try_exists(&base_path).await.unwrap() {
        tokio::fs::create_dir(&base_path).await.unwrap();
    }

    let bot = Bot::from_env()
        .parse_mode(ParseMode::MarkdownV2)
        .into_inner();

    Dispatcher::builder(bot, utils::bot::build_dp_tree())
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
