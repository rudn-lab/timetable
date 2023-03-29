use once_cell::sync::Lazy;
use std::collections::HashMap;
use worker::kv::{KvError, KvStore};

use chrono::{Duration, NaiveTime};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub start_offset: TimeOffset,
    pub duration: ClassDuration,
}

pub type Timetable = HashMap<Day, Vec<Event>>;

/// Offset from the beginning of the day measured in classes 90 minutes each
pub type TimeOffset = f64;
pub type ClassDuration = f64;
