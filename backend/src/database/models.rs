use crate::database::schema::faculties;
use diesel::prelude::*;

#[derive(Queryable, Insertable, Debug)]
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
