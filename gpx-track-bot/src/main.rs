use geo::prelude::*;
use serde_json::Value;
use std::io::BufWriter;
use std::path::Path;
use teloxide::types::InputFile;

use geo::point;
use gpx::{read, write};
use gpx::{Gpx, Track, TrackSegment};
use std::fs::{remove_file, File};
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
        distance: u32,
        start_address: String,
    },
    #[command(
        description = "get the cut out version of the gxp file from coordinates",
        parse_with = "split"
    )]
    CutGPXCoordinates {
        filename: String,
        distance: u32,
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
    expected_distance: u32,
    longitude: f64,
    latitude: f64,
) {
    let location_point = point!(x: longitude, y: latitude);
    println!(
        "handling gpx file {} {} {} {}",
        filename, expected_distance, longitude, latitude
    );
    println!(
        "location_point {} {}",
        location_point.x(),
        location_point.y()
    );
    let file = File::open(format!("gpx_files/{}", filename)).unwrap();
    let reader = BufReader::new(file);

    // read takes any io::Read and gives a Result<Gpx, Error>.
    let gpx_file: Gpx = read(reader).unwrap();
    let track: &Track = &gpx_file.tracks[0];
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
        let mut min_distance: f64 = 100_000_000.0;
        let mut min_index = 0;
        let mut count = 0;
        let mut bigger_count = 0;
        for point in &segment.points {
            // find nearest point in gpx file
            let distance = point.point().geodesic_distance(&location_point);

            if distance < min_distance {
                min_distance = distance;
                min_index = count;
                bigger_count = 0;
            } else {
                bigger_count += 1;
            }

            count += 1;
            if bigger_count > 100 {
                break;
            }
        }

        println!(
            "closest point: {} distance: {} x: {} y: {}",
            min_index,
            min_distance,
            segment.points[min_index].point().x(),
            segment.points[min_index].point().y()
        );

        let mut resulting_gpx: Gpx = gpx::Gpx::default();
        resulting_gpx.version = gpx_file.version;
        resulting_gpx.tracks.clear();

        let mut new_segment: TrackSegment = gpx::TrackSegment::new();
        let mut lenght = 0.0;
        let mut last_point = segment.points[min_index].point();

        for i in min_index..segment.points.capacity() - 1 {
            let current_waypoint = segment.points[i].clone();
            let current_point = current_waypoint.point();
            new_segment.points.push(current_waypoint);
            lenght += current_point.geodesic_distance(&last_point) / 1000.0;
            last_point = current_point;
            println!(
                "lens: {} {}",
                new_segment.linestring().euclidean_length() * 100.0,
                lenght
            );
            if lenght > f64::from(expected_distance) {
                break;
            }
        }

        let mut new_track: Track = gpx::Track::new();
        new_track.segments.push(new_segment);

        resulting_gpx.tracks.push(new_track);

        let temp_file_path = "gpx_files/temp_track.gpx";
        if Path::new(temp_file_path).exists() {
            remove_file(temp_file_path).unwrap();
        }

        let out_file: File = File::create(temp_file_path).unwrap();
        let writer = BufWriter::new(out_file);
        write(&resulting_gpx, writer).unwrap();

        bot.send_document(msg.chat.id, InputFile::file(temp_file_path))
            .await
            .unwrap();
    } else {
        //TODO: return error message that there is no track in the gpx selected / provided
    }
}
