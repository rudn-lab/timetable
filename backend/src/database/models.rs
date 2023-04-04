use crate::database::schema::faculties;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = faculties)]
pub struct Faculty {
    pub uuid: String,
    pub name: String,
}

// #[derive(Insertable, Debug)]
// #[diesel(table_name = faculties)]
// pub struct NewFaculty<'a> {
//     pub uuid: &'a str,
//     pub name: &'a str,
// }
