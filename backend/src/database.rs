use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

pub mod models;
pub mod schema;

pub struct Database {
    pub conn: SqliteConnection,
}

impl Database {
    pub fn new() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {database_url}"));
        Self { conn }
    }

    pub fn update_faculties(&mut self, faculties: &Vec<models::Faculty>) {
        // Insert faculties one by one so that if there's a new faculty at the end of the list it
        // will be added even if all previous throw UniqueViolation errors
        for faculty in faculties {
            match diesel::insert_into(schema::faculties::table)
                .values(faculty)
                .execute(&mut self.conn)
            {
                Ok(_) => log::info!(
                    "Added new Faculty '{}' with uuid: '{}'",
                    faculty.name,
                    faculty.uuid,
                ),
                Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                    log::info!(
                        "Faculty '{}' with uuid '{}' already exists in the database",
                        faculty.name,
                        faculty.uuid
                    )
                }
                Err(msg) => log::error!(
                    "Error: '{}' while inserting faculty '{}' with uuid: '{}'",
                    msg,
                    faculty.name,
                    faculty.uuid
                ),
            }
        }
    }

    /// Returns current faculties of the RUDN university based on selection parameter,
    /// or None if the database is empty or if there is an error
    pub fn get_faculties(&mut self, selection: FacultiesSelection) -> Option<Vec<models::Faculty>> {
        use schema::faculties::dsl::*;
        let query = match selection {
            FacultiesSelection::Total => faculties.load::<models::Faculty>(&mut self.conn),
            FacultiesSelection::Partial(selection) => faculties
                .filter(uuid.eq_any(selection))
                .load::<models::Faculty>(&mut self.conn),
        };
        match query {
            Ok(val) => {
                if !val.is_empty() {
                    Some(val)
                } else {
                    None
                }
            }
            Err(err) => {
                log::error!("{err}");
                None
            }
        }
    }
}

pub enum FacultiesSelection<'a> {
    Total,
    Partial(&'a Vec<models::Uuid>),
}
