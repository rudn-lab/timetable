use crate::database::schema::*;
use chrono::NaiveTime;
use diesel::{prelude::*, sql_types::Text, AsExpression, FromSqlRow};
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

#[derive(
    Eq, PartialEq, Hash, Clone, Copy, Debug, Serialize, Deserialize, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = Text)]
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
    pub student_group: Uuid,
}

#[derive(Insertable, Clone, Debug, Serialize, Deserialize)]
#[diesel(table_name = timetables)]
pub struct InsertableEvent {
    pub name: String,
    pub day: String,
    pub start_time: String,
    pub end_time: String,
    pub student_group: Uuid,
}

impl From<Event> for InsertableEvent {
    fn from(value: Event) -> Self {
        Self {
            name: value.name,
            day: serde_json::to_string(&value.day).unwrap(),
            start_time: value.start_time.format("%H:%M").to_string(),
            end_time: value.end_time.format("%H:%M").to_string(),
            student_group: value.student_group,
        }
    }
}

impl Queryable<timetables::SqlType, diesel::sqlite::Sqlite> for Event {
    type Row = (i32, String, String, String, String, String);

    fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
        Ok(Self {
            name: row.1,
            day: serde_json::from_str(&row.2).unwrap(),
            start_time: NaiveTime::parse_from_str(&row.3, "%H:%M").unwrap(),
            end_time: NaiveTime::parse_from_str(&row.4, "%H:%M").unwrap(),
            student_group: row.5,
        })
    }
}
