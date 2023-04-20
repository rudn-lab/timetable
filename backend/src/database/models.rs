use crate::database::schema::*;
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
