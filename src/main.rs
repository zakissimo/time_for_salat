use anyhow::Result;
use chrono::Local;
use prayer_times::PrayerTimes;
use reqwest::blocking::Client;
use select::document::Document;
use select::predicate::Name;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const FILE_PATH: &str = "/tmp/time_4_salat.log";

fn fetch_data(url: &str) -> Result<reqwest::blocking::Response, String> {
    let response = Client::new()
        .get(url)
        .header(reqwest::header::CACHE_CONTROL, "no-cache")
        .send()
        .map_err(|e| format!("Failed to send request: {}", e))?;

    Ok(response)
}

fn parse_data(doc: Document) -> Result<PrayerTimes, String> {
    for element in doc.find(Name("script")) {
        if element.inner_html().contains("var confPrayerTimes") {
            let line = element.inner_html();
            let lines = line.split(';');
            for line in lines {
                let json_string = line
                    .trim()
                    .trim_start_matches("var confPrayerTimes = ")
                    .trim_end_matches(';');
                if json_string.starts_with('{') {
                    let data: Result<PrayerTimes, serde_json::Error> =
                        serde_json::from_str(json_string);
                    match data {
                        Ok(data) => return Ok(data),
                        Err(_) => return Err("Can't parse html response".to_string()),
                    }
                }
            }
            return Err("Couldn't parse html response!".to_string());
        }
    }
    Err("Couldn't parse html response!".to_string())
}

fn fetch_and_parse_data(url: &str) -> Option<PrayerTimes> {
    let r = fetch_data(url).ok()?;
    let body = r.text().ok()?;
    let doc = Document::from(body.as_str());
    let data = parse_data(doc).ok()?;

    Some(data)
}

fn should_update_file() -> bool {
    if !Path::new(FILE_PATH).exists() {
        return true;
    }

    let now = Local::now().format("%d/%m/%y").to_string();
    if let Ok(file) = File::open(FILE_PATH) {
        let reader = BufReader::new(file);
        if let Some(Ok(last_modified)) = reader.lines().next() {
            return now != last_modified;
        }
        return true;
    }
    eprintln!("Error: Can't open the file.");

    false
}

fn get_data_from_file() -> Option<PrayerTimes> {
    let content = fs::read_to_string(FILE_PATH)
        .ok()?
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");

    serde_json::from_str(&content).expect("String content should match PrayerTimes object")
}

fn main() {
    let env_var = "MWQT";

    match env::var(env_var) {
        Ok(var) => {
            let mut data = PrayerTimes::default();
            data.file_path = FILE_PATH.to_string();

            if should_update_file() {
                if let Some(data) = fetch_and_parse_data(&var) {
                    if let Err(err) = data.update() {
                        eprintln!("Error: Can't update file: {}", err);
                    }
                } else {
                    eprintln!("Error: Failed to fetch and parse data.");
                }
            }
        }
        Err(_) => {
            println!("Environment variable {} is not set", env_var);
            return;
        }
    }

    if let Some(mut data) = get_data_from_file() {
        data.file_path = FILE_PATH.to_string();
        let remaining_time = data.get_remaining_time();
        println!("{}", remaining_time);
    } else {
        eprintln!("Error: Failed to get data from file.");
    }
}
