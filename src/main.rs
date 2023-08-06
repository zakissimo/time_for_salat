use reqwest::blocking::Client;
use chrono::{NaiveTime, Local};
use select::document::Document;
use select::predicate::Name;
use serde::Deserialize;
use serde_json;
use std::fmt;
// use chrono::DateTime;

fn fetch() -> Result<reqwest::blocking::Response, reqwest::Error> {
    let url = "https://mawaqit.net/fr/m-angouleme";
    // let url = "https://mawaqit.net/fr/mosquee-dagen";

    let client = Client::new();
    return client.get(url).send();
}

#[derive(Debug, Deserialize)]
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
                    let data : Result<Data, serde_json::Error> = serde_json::from_str(json_string);
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

fn get_remaining_time(data: Data) -> String {
    let now = Local::now().time();

    let remaining_time = data
        .times
        .into_iter()
        .filter_map(|time| NaiveTime::parse_from_str(&time, "%H:%M").ok()) // Filter out invalid times
        .filter(|&time| time > now) // Filter times that are greater than the current time
        .fold(None, |closest_time, time| {
            // Calculate the closest time to the current time
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
            format!("{}:{:02}", duration.num_hours(), duration.num_minutes() % 60)
        }
        None => "??:??".to_string(),
    }
}

fn main() {
    let r = fetch();
    match r {
        Ok(r) => match r.text() {
            Ok(body) => {
                let doc = Document::from(body.as_str());
                let data = parse(doc);
                match data {
                    Ok(data) =>  {
                        let s = get_remaining_time(data);
                        println!("{s}");

                    },
                    Err(_) => eprintln!("Error while parsing the document."),
                }
            },
            Err(_) => eprintln!("Error while decoding the response to text."),
        },
        Err(_) => eprintln!("Error while sending request."),
    };
}
