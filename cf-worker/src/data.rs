use std::collections::HashMap;

use chrono::NaiveTime;
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

// pub const CLASS_DURATION: chrono::Duration = chrono::Duration::minutes(90);

pub fn class_begin_times() -> [NaiveTime; 8] {
    [
        NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        NaiveTime::from_hms_opt(13, 30, 0).unwrap(),
        NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
        NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
        NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        NaiveTime::from_hms_opt(19, 30, 0).unwrap(),
    ]
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub offset: ClassOffset,
}

pub type Timetable = HashMap<Day, Vec<Event>>;

/// Offset from the beginning of the day measured in classes 90 minutes each
pub type ClassOffset = f64;
