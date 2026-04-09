use chrono::{Local, NaiveTime};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct PrayerTimes {
    pub name: String,
    pub slug: String,
    pub times: [String; 6], // fajr, shuruq, dhuhr, asr, maghrib, isha
}

impl PrayerTimes {
    pub fn from_file(file_path: &str, skip_header: bool) -> Option<Self> {
        let content: String = fs::read_to_string(file_path)
            .ok()?
            .lines()
            .skip(if skip_header { 1 } else { 0 })
            .collect::<Vec<_>>()
            .join("\n");

        serde_json::from_str(&content).ok()
    }

    pub fn update(&self, file_path: &str) -> Result<(), std::io::Error> {
        let now = Local::now().format("%d/%m/%Y").to_string();
        let content = now + "\n" + &serde_json::to_string(&self)?;

        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file_path)?
            .write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn get_remaining_time(&self) -> String {
        let now = Local::now().time();

        let next = self
            .times
            .iter()
            .filter_map(|t| {
                let parsed = NaiveTime::parse_from_str(t, "%H:%M").ok()?;
                if parsed > now {
                    Some((t.as_str(), parsed))
                } else {
                    None
                }
            })
            .next();

        match next {
            Some((time_str, parsed)) => {
                let total_minutes = (parsed - now).num_minutes();
                let duration = if total_minutes >= 60 {
                    format!("{}h{:02}m", total_minutes / 60, total_minutes % 60)
                } else {
                    format!("{}m", total_minutes)
                };
                format!("{} ({})", time_str, duration)
            }
            None => format!("{} (fajr)", &self.times[0]),
        }
    }
}
