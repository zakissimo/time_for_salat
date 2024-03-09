use anyhow::Result;
use chrono::Datelike;
use chrono::{Local, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct PrayerTimes {
    name: String,
    times: [String; 5],
    calendar: [HashMap<String, Vec<String>>; 12],
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub file_path: String,
}

impl PrayerTimes {
    pub fn update(&self) -> Result<(), std::io::Error> {
        let now = Local::now().format("%d/%m/%y").to_string();
        let content = now + "\n" + &serde_json::to_string(&self)?;

        OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.file_path)?
            .write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn get_remaining_time(&self) -> String {
        let now = Local::now().time();

        let day = Utc::now().day();
        let day_str = format!("{:}", day);
        let month = Utc::now().month();
        let times = self.calendar[month as usize - 1]
            .get(&day_str)
            .expect("The day should be in the calendar");

        let remaining_time = times
            .clone()
            .into_iter()
            .filter_map(|time| NaiveTime::parse_from_str(&time, "%H:%M").ok()) // Filter out invalid times
            .filter(|&time| time > now)
            .collect::<Vec<NaiveTime>>();

        if remaining_time.is_empty() {
            return times[0].to_owned();
        }
        let duration = remaining_time[0] - now;
        format!(
            "{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60
        )
    }
}

impl fmt::Display for PrayerTimes {
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
