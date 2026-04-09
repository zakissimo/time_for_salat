use chrono::Local;
use prayer_times::PrayerTimes;
use reqwest::blocking::Client;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const FILE_PATH: &str = "/tmp/time_for_salat.log";
const API_URL: &str = "https://mawaqit.net/api/2.0/mosque/search";

fn fetch_and_parse_data(slug: &str) -> Option<PrayerTimes> {
    let url = format!("{}?word={}", API_URL, slug);
    let response = Client::new()
        .get(&url)
        .send()
        .ok()?
        .json::<Vec<serde_json::Value>>()
        .ok()?;

    let entry = response.iter().find(|e| e["slug"].as_str() == Some(slug))?;

    let name = entry["name"].as_str()?.to_string();
    let times_vec: Vec<String> = entry["times"]
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let times: [String; 6] = times_vec.try_into().ok()?;

    Some(PrayerTimes {
        name,
        slug: slug.to_string(),
        times,
    })
}

fn should_update_file(slug: &str) -> bool {
    if !Path::new(FILE_PATH).exists() {
        return true;
    }

    let now = Local::now().format("%d/%m/%Y").to_string();
    if let Ok(file) = File::open(FILE_PATH) {
        let reader = BufReader::new(file);
        if let Some(Ok(last_modified)) = reader.lines().next() {
            if now != last_modified {
                return true;
            }
        } else {
            return true;
        }
    }

    match PrayerTimes::from_file(FILE_PATH, true) {
        Some(cached) => cached.slug != slug,
        None => true,
    }
}

fn main() {
    let slug = env::args().nth(1).or_else(|| env::var("MWQT").ok());
    let slug = match slug {
        Some(s) => s,
        None => {
            eprintln!("Usage: time_for_salat <slug> or set MWQT env var");
            return;
        }
    };

    if should_update_file(&slug) {
        if let Some(data) = fetch_and_parse_data(&slug) {
            if let Err(err) = data.update(FILE_PATH) {
                eprintln!("Error: Can't update file: {}", err);
            }
        } else {
            eprintln!("Error: Failed to fetch and parse data.");
        }
    }

    if let Some(data) = PrayerTimes::from_file(FILE_PATH, true) {
        println!("{}", data.get_remaining_time());
    } else {
        eprintln!("Error: Failed to get data from file.");
    }
}
