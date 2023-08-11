use chrono::{Local, NaiveTime};
use reqwest::blocking::Client;
use select::document::Document;
use select::predicate::Name;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    times: [String; 5],
    name: String,
    localisation: String,
    email: String,
    jumua: String,
    shuruq: String,
}

static FILE_PATH: &str = "/dev/shm/Time4Salat.log";

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Data:\n\
            Times: {:?}\n\
            Name: {}\n\
            Localisation: {}\n\
            Email: {}\n\
            Jumua: {}\n\
            Shuruq: {}",
            self.times, self.name, self.localisation, self.email, self.jumua, self.shuruq
        )
    }
}

fn fetch_data() -> Result<reqwest::blocking::Response, reqwest::Error> {
    let url = "https://mawaqit.net/fr/m-angouleme";
    // let url = "https://mawaqit.net/fr/mosquee-dagen";

    let client = Client::new();
    return client.get(url).send();
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
                    .trim_end_matches(";");
                if json_string.starts_with("{") {
                    let data: Result<Data, serde_json::Error> = serde_json::from_str(json_string);
                    match data {
                        Ok(data) => return Ok(data),
                        Err(_) => return Err("Couldn't parse html response!".to_string()),
                    }
                }
            }
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

fn get_remaining_time(data: Data) -> String {
    let now = Local::now().time();

    let remaining_time = data
        .times
        .into_iter()
        .filter_map(|time| NaiveTime::parse_from_str(&time, "%H:%M").ok()) // Filter out invalid times
        .filter(|&time| time > now)
        .fold(None, |closest_time, time| match closest_time {
            Some(prev_time) => {
                if time < prev_time {
                    Some(time)
                } else {
                    Some(prev_time)
                }
            }
            None => Some(time),
        });

    match remaining_time {
        Some(time) => {
            let duration = time.signed_duration_since(now);
            format!(
                "{}:{:02}",
                duration.num_hours(),
                duration.num_minutes() % 60
            )
        }
        None => "??:??".to_string(),
    }
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
        eprintln!("Error opening the file.")
    }

    false
}

fn get_data_from_file() -> Option<Data> {
    let mut content = fs::read_to_string(FILE_PATH).ok()?;

    let mut lines = content.lines();
    lines.next();

    content = lines.collect::<Vec<_>>().join("\n");
    match serde_json::from_str(&content) {
        Ok(data) => Some(data),
        Err(err) => {
            eprintln!("Error parsing JSON: {}", err);
            None
        }
    }
}

fn update_data_file(data: &Data) -> Result<(), std::io::Error> {
    let now = Local::now().format("%d/%m/%y").to_string();
    let mut content = now + "\n";
    content += &serde_json::to_string(&data).unwrap();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(FILE_PATH)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

fn main() {
    if should_update_file() {
        if let Some(data) = fetch_and_parse_data() {
            if let Err(err) = update_data_file(&data) {
                eprintln!("Error updating file: {}", err);
            }
        } else {
            eprintln!("Error: Failed to fetch and parse data.");
        }
    }

    if let Some(data) = get_data_from_file() {
        let remaining_time = get_remaining_time(data);
        println!("{}", remaining_time);
    } else {
        eprintln!("Error: Failed to get data from file.");
    }
}
