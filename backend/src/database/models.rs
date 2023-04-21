use crate::database::schema::*;
use chrono::NaiveTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

pub type Uuid = String;

#[derive(Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = faculties)]
pub struct Faculty {
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Queryable, Insertable, Clone, Debug, Serialize, Deserialize)]
#[diesel(table_name = groups)]
pub struct Group {
    pub uuid: Uuid,
    pub name: String,
    pub faculty: Uuid,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

pub enum DayError {
    InvalidRussianWeekdayName,
}

impl Day {
    pub fn from_russian(name: &str) -> std::result::Result<Self, DayError> {
        match name.to_lowercase().as_str() {
            "понедельник" => Ok(Self::Monday),
            "вторник" => Ok(Self::Tuesday),
            "среда" => Ok(Self::Wednesday),
            "четверг" => Ok(Self::Thursday),
            "пятница" => Ok(Self::Friday),
            "суббота" => Ok(Self::Saturday),
            _ => {
                log::error!("Invalid russian weekday: {name}");
                Err(DayError::InvalidRussianWeekdayName)
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub day: Day,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}
