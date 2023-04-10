use diesel::connection::DefaultLoadingMode;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::collections::HashMap;
use std::env;

pub mod models;
use models::*;
pub mod schema;

pub enum DBError {
    ForeignKeyError,
}

/// Insert elements one by one so that if there's a new one at the end of the list it
/// will be added even if all previous throw UniqueViolation errors
macro_rules! update_table {
    ($conn:expr, $table:expr, $aggregate:expr) => {{
        let mut res = Ok(());
        for entry in $aggregate {
            match diesel::insert_into($table).values(entry).execute($conn) {
                // Todo: find a way to set table name in logs
                Ok(_) => log::info!("Added new entry '{:?}' into table '{:?}'", entry, $table),
                Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                    log::warn!(
                        "Skipping: entry '{:?}' already exists in table '{:?}'",
                        entry,
                        $table
                    )
                }
                Err(DatabaseError(DatabaseErrorKind::ForeignKeyViolation, info)) => {
                    log::error!("{info:?}");
                    res = Err(DBError::ForeignKeyError);
                    break;
                }
                Err(msg) => log::error!(
                    "Error: '{}' while inserting entry '{:?}' into table '{:?}'",
                    msg,
                    entry,
                    $table
                ),
            }
        }
        res
    }};
}

pub struct Database {
    pub conn: SqliteConnection,
}

impl Database {
    pub fn new() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let mut conn = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {database_url}"));
        match diesel::sql_query("PRAGMA foreign_keys = ON").execute(&mut conn) {
            Ok(_) => log::info!("Activated foreign keys in database"),
            Err(e) => {
                log::error!("Could not activate foreigh keys in database: '{e}'")
            }
        };
        Self { conn }
    }

    pub fn update_faculties(&mut self, new_faculties: &Vec<Faculty>) {
        let _: Result<(), DBError> =
            update_table!(&mut self.conn, schema::faculties::table, new_faculties);
    }

    /// Returns all current faculties of the RUDN university
    /// If the vector is empty, something is wrong with the database
    pub fn get_faculties(&mut self) -> Vec<Faculty> {
        use schema::faculties::dsl::*;
        faculties.load::<Faculty>(&mut self.conn).unwrap_or(vec![])
    }

    pub fn update_groups(&mut self, new_groups: &Vec<Group>) -> Result<(), DBError> {
        update_table!(&mut self.conn, schema::groups::table, new_groups)
    }

    pub fn get_groups_by_faculty(&mut self, faculties: &Vec<Uuid>) -> HashMap<Uuid, Group> {
        use schema::groups::dsl::*;
        match groups
            .filter(faculty.eq_any(faculties))
            .load_iter::<Group, DefaultLoadingMode>(&mut self.conn)
        {
            Ok(query_res) => query_res
                .map(|el| {
                    let el = el.unwrap();
                    (el.faculty.clone(), el)
                })
                .collect::<HashMap<Uuid, Group>>(),
            Err(e) => {
                log::error!("{e:?}");
                HashMap::new()
            }
        }
    }
}
