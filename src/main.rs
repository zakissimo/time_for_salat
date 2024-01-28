use anyhow::Result;
use chrono::{Local, NaiveTime};
use reqwest::blocking::Client;
use select::document::Document;
use select::predicate::Name;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    times: [String; 5],
    name: String,
}

impl Data {
    fn update(&self) -> Result<(), std::io::Error> {
        let now = Local::now().format("%d/%m/%y").to_string();
        let content = now + "\n" + &serde_json::to_string(&self)?;

        OpenOptions::new()
            .write(true)
            .create(true)
            .open(FILE_PATH)?
            .write_all(content.as_bytes())?;

        Ok(())
    }

    fn get_remaining_time(self) -> String {
        let now = Local::now().time();

        let remaining_time = self
            .times
            .clone()
            .into_iter()
            .filter_map(|time| NaiveTime::parse_from_str(&time, "%H:%M").ok()) // Filter out invalid times
            .filter(|&time| time > now)
            .collect::<Vec<NaiveTime>>();

        if remaining_time.is_empty() {
            return self.times[0].to_owned();
        }
        let duration = remaining_time[0] - now;
        format!(
            "{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60
        )
    }
}

const FILE_PATH: &str = "/dev/shm/Time4Salat.log";

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Data:\n\
            Times: {:?}\n\
            Name: {}",
            self.times, self.name,
        )
    }
}

fn fetch_data() -> Result<reqwest::blocking::Response, String> {
    let env_var = "MWQT";

    let url =
        env::var(env_var).map_err(|_| format!("{} environment variable is not set", env_var))?;

    let response = Client::new()
        .get(url)
        .send()
        .map_err(|e| format!("Failed to send request: {}", e))?;

    Ok(response)
}

fn parse_data(doc: Document) -> Result<Data, String> {
    for element in doc.find(Name("script")) {
        if element.inner_html().contains("var confData") {
            let line = element.inner_html();
            let lines = line.split(';');
            for line in lines {
                let json_string = line
                    .trim()
                    .trim_start_matches("var confData = ")
                    .trim_end_matches(';');
                if json_string.starts_with('{') {
                    let data: Result<Data, serde_json::Error> = serde_json::from_str(json_string);
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

fn fetch_and_parse_data() -> Option<Data> {
    let r = fetch_data().ok()?;
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
    } else {
        eprintln!("Error: Can't open the file.");
    }

    false
}

fn get_data_from_file() -> Option<Data> {
    let content = fs::read_to_string(FILE_PATH)
        .ok()?
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");

    serde_json::from_str(&content).expect("String content should match Data object")
}

fn main() {
    if should_update_file() {
        if let Some(data) = fetch_and_parse_data() {
            if let Err(err) = data.update() {
                eprintln!("Error: Can't update file: {}", err);
            }
        } else {
            eprintln!("Error: Failed to fetch and parse data.");
        }
    }

    if let Some(data) = get_data_from_file() {
        let remaining_time = data.get_remaining_time();
        println!("{}", remaining_time);
    } else {
        eprintln!("Error: Failed to get data from file.");
    }
}
