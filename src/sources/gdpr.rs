use csv::Reader;
use jiff::Zoned;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use std::path::Path;

pub trait Gdpr {
    const FILENAME: &'static str;
}

pub mod date {
    use super::*;
    use jiff::{civil::DateTime, tz::TimeZone};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Zoned, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Parse format like "2017-04-24 18:52:15 UTC"

        // Split the string to separate datetime and timezone
        let parts: Vec<&str> = s.rsplitn(2, ' ').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom(
                format!("Date string '{}' missing timezone (expected format: 'YYYY-MM-DD HH:MM:SS TZ')", s)
            ));
        }

        let (tz_str, datetime_str) = (parts[0], parts[1]);

        // Verify timezone is UTC
        if tz_str != "UTC" {
            return Err(serde::de::Error::custom(
                format!("Expected UTC timezone but got: {} (in date string: '{}')", tz_str, s)
            ));
        }

        // Parse the datetime portion
        let dt = datetime_str.parse::<DateTime>()
            .map_err(|e| serde::de::Error::custom(
                format!("Failed to parse datetime from '{}': {}", datetime_str, e)
            ))?;

        // Convert to zoned datetime with UTC timezone
        dt.to_zoned(TimeZone::UTC).map_err(serde::de::Error::custom)
    }
}

pub fn list<T>(export_dir: &Path) -> impl Iterator<Item = T>
where
    T: Gdpr + DeserializeOwned,
{
    let mut p = export_dir.to_path_buf();
    p.push(T::FILENAME);

    let things = Reader::from_path(p).unwrap();
    things.into_deserialize().map(|f| f.unwrap())
}
