use geo::prelude::*;
use serde_json::{Result, Value};
use std::fmt::format;

use geo::geodesic_distance;
use geo::point;
use gpx::read;
use gpx::{Gpx, Track, TrackSegment};
use std::fs::File;
use std::io::BufReader;
use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        dptree::entry()
            .branch(
                Update::filter_message()
                    .filter_command::<Command>()
                    .endpoint(answer),
            )
            .branch(Update::filter_message().endpoint(message_handler)),
    )
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "roll a dice")]
    Dice,
    #[command(
        description = "get the cut out version of the gxp file",
        parse_with = "split"
    )]
    CutGPX {
        filename: String,
        distance: u64,
        start_address: String,
    },
    #[command(
        description = "get the cut out version of the gxp file from coordinates",
        parse_with = "split"
    )]
    CutGPXCoordinates {
        filename: String,
        distance: u64,
        longitude: f64,
        latitude: f64,
    },
}

// add command to get new gpx file
// get closest point first
// initially read gpx file from local storage
// future support multiple files
// add user check (permission wise)

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Dice => {
            bot.send_dice(msg.chat.id).await?;
        }
        Command::CutGPX {
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
            println!("url {}", url);
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
            println!("lon {} lat {}", lon, lat);
            handle_gpx_file(bot, msg, filename, distance, lon, lat).await;
        }
        Command::CutGPXCoordinates {
            filename,
            distance,
            longitude,
            latitude,
        } => {
            bot.send_message(msg.chat.id, "preparing gpx please wait")
                .await?;
            handle_gpx_file(bot, msg, filename, distance, longitude, latitude).await;
        }
    };
    Ok(())
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

async fn handle_gpx_file(
    bot: Bot,
    msg: Message,
    filename: String,
    distance: u64,
    longitude: f64,
    latitude: f64,
) {
    let location_point = point!(x: latitude, y: longitude);
    println!(
        "handling gpx file {} {} {} {}",
        filename, distance, longitude, latitude
    );
    let file = File::open("gpx_files/EuroVelo15developed.gpx").unwrap();
    let reader = BufReader::new(file);

    // read takes any io::Read and gives a Result<Gpx, Error>.
    let gpx: Gpx = read(reader).unwrap();
    let track: &Track = &gpx.tracks[0];
    match &track.name {
        Some(name) => {
            println!("track name: {}", name);
        }
        None => {
            println!("track has no name");
        }
    }
    if track.segments.capacity() > 0 {
        let segment: &TrackSegment = &track.segments[0];
        println!("segment points {}", segment.points.capacity());
        let mut minDistance = 100_000_000;
        let mut min_index = 0;
        let mut count = 0;
        let mut bigger_count = 0;
        for point in &segment.points {
            // find nearest point in gpx file
            let distance = point.point().geodesic_distance(&location_point);

            println!(
                "index {} wp: x:{} y:{} distance: {}",
                count,
                point.point().x(),
                point.point().y(),
                distance
            );
            if distance < minDistance {
                distance = minDistance;
                min_index = count;
                bigger_count = 0;
            } else {
                bigger_count += 1;
            }

            count += 1;
            if bigger_count > 10 {
                break;
            }
        }

        // create new gpx file with the right starting point and return it in the chat
    } else {
        //TODO: return error message that there is no track in the gpx selected / provided
    }

    // create new gpx file for the new distance
    // TODO: send new file to chat
    // bot.send_message(msg.chat.id, "here is your gpx have fun")
    // .await?;
}
