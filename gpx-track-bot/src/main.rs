mod utils;

use std::env;
use teloxide::types::ParseMode;

use teloxide::prelude::*;

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

    let bot = Bot::from_env()
        .parse_mode(ParseMode::MarkdownV2)
        .into_inner();

    Dispatcher::builder(bot, utils::bot::build_dp_tree())
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

// future support multiple files
