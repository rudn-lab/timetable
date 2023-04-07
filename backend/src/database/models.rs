use crate::database::schema::faculties;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

pub type Uuid = String;

#[derive(Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = faculties)]
pub struct Faculty {
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub uuid: Uuid,
    pub name: String,
    pub faculty: Uuid,
}
