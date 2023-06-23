use reqwest::blocking::Client;
use scraper::{Html, Selector};

fn fetch() -> String {
    let url = "https://mawaqit.net/fr/m-angouleme";
    // let url = "https://mawaqit.net/fr/mosquee-dagen";

    let client = Client::new();
    let r = client.get(url).send();
    match r {
        Ok(r) => match r.text(){
            Ok(body) => body,
            Err(_) => "Error while decoding the response to text.".to_string()
        },
        Err(_) => "Error while sending request.".to_string()
    }
}

fn main() {
    let body = fetch();
    // println!("{}", body);
    let doc = Html::parse_document(&body);
    let selector = Selector::parse(r#"script"#).unwrap();

    for element in doc.select(&selector) {
        println!("{}", element.inner_html())
    }
}
