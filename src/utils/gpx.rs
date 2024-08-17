use geo::prelude::*;
use rand::distributions::{Alphanumeric, DistString};
use std::env;
use std::io::BufWriter;
use std::path::Path;

use geo::point;
use gpx::{read, write};
use gpx::{Gpx, Track, TrackSegment};
use std::fs::{remove_file, File};
use std::io::BufReader;

pub fn generate_partial_gpx(
    filename: String,
    expected_distance: u32,
    longitude: f64,
    latitude: f64,
) -> Option<String> {
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

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    // read takes any io::Read and gives a Result<Gpx, Error>.
    let gpx_file: Gpx = read(reader).unwrap();
    let track: &Track = &gpx_file.tracks[0];
    // TODO: extend to also set a track name (optional)
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

            if lenght > f64::from(expected_distance) {
                break;
            }
        }

        let mut new_track: Track = gpx::Track::new();
        new_track.segments.push(new_segment);

        resulting_gpx.tracks.push(new_track);
        let temp_file_path = format!(
            "{}/temp_track{}.gpx",
            env::var("GPX_TRACK_BOT_DATA").unwrap_or("/gpx_files".to_string()),
            Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
        );
        if Path::new(&temp_file_path).exists() {
            remove_file(&temp_file_path).unwrap();
        }

        let out_file: File = File::create(&temp_file_path).unwrap();
        let writer = BufWriter::new(out_file);
        write(&resulting_gpx, writer).unwrap();

        return Some(temp_file_path);
    } else {
        return None;
        //TODO: return error message that there is no track in the gpx selected / provided / matching the query
    }
}
