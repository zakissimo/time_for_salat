use chrono::{DateTime, Local, NaiveTime};
use reqwest::blocking::Client;
use select::document::Document;
use select::predicate::Name;
use serde::{Serialize, Deserialize};
use serde_json;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    times: [String; 5],
    name: String,
    localisation: String,
    email: String,
    jumua: String,
    jumuaAsDuhr: bool,
    shuruq: String,
}

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
            Jumua as Duhr: {}\n\
            Shuruq: {}",
            self.times,
            self.name,
            self.localisation,
            self.email,
            self.jumua,
            self.jumuaAsDuhr,
            self.shuruq
        )
    }
}

fn fetch() -> Result<reqwest::blocking::Response, reqwest::Error> {
    let url = "https://mawaqit.net/fr/m-angouleme";
    // let url = "https://mawaqit.net/fr/mosquee-dagen";

    let client = Client::new();
    return client.get(url).send();
}

fn parse(doc: Document) -> Result<Data, String> {
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

fn fetch_and_parse() -> Option<Data> {
    let r = fetch().ok()?;
    let body = r.text().ok()?;
    let doc = Document::from(body.as_str());
    let data = parse(doc).ok()?;

    Some(data)
}

fn get_remaining_time(data: Data) -> String {
    let now = Local::now().time();

    let remaining_time = data
        .times
        .into_iter()
        .filter_map(|time| NaiveTime::parse_from_str(&time, "%H:%M").ok()) // Filter out invalid times
        .filter(|&time| time > now)
        .fold(None, |closest_time, time| {
            match closest_time {
                Some(prev_time) => {
                    if time < prev_time {
                        Some(time)
                    } else {
                        Some(prev_time)
                    }
                }
                None => Some(time),
            }
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

fn file_exists(file_path: &str) -> bool {
    Path::new(file_path).exists()
}

fn get_last_modified(file_path: &str) -> Option<DateTime<Local>> {
    fs::metadata(file_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok().map(|modified| modified.into()))
}

fn build_file() {
    let file_path = "/tmp/Time4Salat.log";
    let data = fetch_and_parse();

    if let Some(data) = data {
        let content = serde_json::to_string(&data).unwrap();
        if let Err(err) = fs::write(file_path, content) {
            eprintln!("Error writing to file: {}", err);
        }
    } else {
        eprintln!("Error: No data available.");
    }
}

fn get_data() -> Option<Data> {
    let file_path = "/tmp/Time4Salat.log";
    if !file_exists(file_path) {
        build_file();
    } else {
        let last_modified = get_last_modified(file_path);
        match last_modified {
            Some(last_modified) => {
                let now = Local::now().format("%d/%m/%y").to_string();
                let last_modified = last_modified.format("%d/%m/%y").to_string();
                if now != last_modified {
                    build_file();
                }
            }
            None => eprintln!("Couldn't get last modified date of file."),
        }
    }
    let content = fs::read_to_string(file_path).ok()?;
    match serde_json::from_str(&content) {
    Ok(data) => Some(data),
    Err(err) => {
        eprintln!("Error parsing JSON: {}", err);
        None
    }
}
}

fn main() {
    println!("{}", get_remaining_time(get_data().unwrap()));
}

// fn main() {
//     let file_path = "/tmp/Time4Salat.log";
//
//     if should_update_file(file_path) {
//         if let Some(data) = fetch_and_parse_data() {
//             if let Err(err) = update_data_file(&data) {
//                 eprintln!("Error updating file: {}", err);
//             }
//         } else {
//             eprintln!("Error: Failed to fetch and parse data.");
//         }
//     }
//
//     if let Some(data) = get_data_from_file(file_path) {
//         let remaining_time = get_remaining_time(data);
//         println!("{}", remaining_time);
//     } else {
//         eprintln!("Error: Failed to get data from file.");
//     }
// }
