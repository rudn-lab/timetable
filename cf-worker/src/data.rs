use once_cell::sync::Lazy;
use std::collections::HashMap;

use chrono::{Duration, NaiveTime};
use serde::{ser::SerializeStruct, Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Day {
    pub fn values() -> [Self; 6] {
        [
            Self::Monday,
            Self::Tuesday,
            Self::Wednesday,
            Self::Thursday,
            Self::Friday,
            Self::Saturday,
        ]
    }
}

/// Class duration is 90 minutes including the break
pub static CLASS_DURATION: Lazy<Duration, fn() -> Duration> = Lazy::new(|| Duration::minutes(90));
/// Start time of the first class
pub static FIRST_CLASS_START: Lazy<NaiveTime, fn() -> NaiveTime> =
    Lazy::new(|| NaiveTime::from_hms_opt(9, 0, 0).unwrap());

#[derive(Debug, PartialEq, Deserialize)]
pub struct Event {
    pub name: String,
    pub start_offset: TimeOffset,
    pub duration: ClassDuration,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Color", 3)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("start_offset", &self.start_offset)?;
        state.serialize_field("duration", &self.duration)?;
        state.serialize_field("start_time", &self.start_time.format("%H:%M").to_string())?;
        state.serialize_field("end_time", &self.end_time.format("%H:%M").to_string())?;

        state.end()
    }
}

pub type Timetable = HashMap<Day, Vec<Event>>;

/// Offset from the beginning of the day measured in classes 90 minutes each
pub type TimeOffset = f64;
pub type ClassDuration = f64;
