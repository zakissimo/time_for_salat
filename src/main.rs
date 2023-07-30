use reqwest::blocking::Client;
use select::document::Document;
use select::predicate::Name;
use serde::Deserialize;
use serde_json;
use std::fmt;
// use chrono::DateTime;

fn fetch() -> String {
    let url = "https://mawaqit.net/fr/m-angouleme";
    // let url = "https://mawaqit.net/fr/mosquee-dagen";

    let client = Client::new();
    let r = client.get(url).send();
    match r {
        Ok(r) => match r.text() {
            Ok(body) => body,
            Err(_) => "Error while decoding the response to text.".to_string(),
        },
        Err(_) => "Error while sending request.".to_string(),
    }
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

impl Data {
    fn default() -> Data {
        Data {
            times: Default::default(),
            name: Default::default(),
            localisation: Default::default(),
            email: Default::default(),
            jumua: Default::default(),
            jumuaAsDuhr: Default::default(),
            shuruq: Default::default(),
        }
    }
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

fn parse(doc: Document) -> Data {
    let mut data: Data = Data::default();
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
                    data = serde_json::from_str(json_string).unwrap();
                }
            }
        }
    }
    data
}

fn main() {
    let body = fetch();
    let doc = Document::from(body.as_str());
    let data: Data = parse(doc);

    println!("{data}");
}
