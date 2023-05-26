use diesel::connection::DefaultLoadingMode;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt::Display;

pub mod models;
use models::*;
pub mod schema;

macro_rules! update_table {
    ($conn:expr, $table:expr, $aggregate:expr) => {{
        match diesel::insert_or_ignore_into($table)
            .values($aggregate)
            .execute($conn)
        {
            Ok(_) => {
                log::debug!(
                    "Added {} new entries into table '{:?}'",
                    $aggregate.len(),
                    $table
                );
                Ok(())
            }
            Err(msg) => {
                log::error!(
                    "Error: '{}' while inserting entries into table '{:?}'",
                    msg,
                    $table
                );
                Err(DBError::UpdateError(format!(
                    "Could not insert data into '{:?}'",
                    $table
                )))
            }
        }
    }};
}

macro_rules! get_filtered_table_vec_data {
    ($conn:expr, $table:expr, $output:ident, ($key_type:ident, $key_field:ident), [$( ( $filter:ident, $val:expr ) ),*]) => {{
        match $table.filter(
            $(
                $filter.eq($val)
            )*
        ).load_iter::<$output, DefaultLoadingMode>($conn) {
            Ok(query_res) => {
                let output = query_res.fold(HashMap::new(), |mut map: HashMap<$key_type, Vec<$output>>, el| {
                    let el = el.unwrap();
                    map.entry(el.$key_field.clone())
                        .and_modify(|list| list.push(el.clone()))
                        .or_insert_with(|| vec![el]);
                    map
                });

                Ok(output)
            }
            Err(msg) => {

                let mut filters = String::from("[ ");
                $(
                    filters.push_str(&format!("{:?} ", $filter));
                )*
                filters.push_str(" ]");

                log::error!(
                    "Error: '{}' while retrieving entries from table '{:?}' with filters: {:?}",
                    msg,
                    $table,
                    filters
                );

                log::warn!("Could not get data from '{:?}' for filters: {:?}", $table, filters);
                Err(DBError::RetrieveError(format!(
                    "Could not retreive data from '{:?}'",
                    $table
                )))
            }
        }
    }};
}

#[derive(Debug)]
pub enum DBError {
    UpdateError(String),
    RetrieveError(String),
}

impl Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for DBError {}

pub type DBResult<T> = Result<T, DBError>;

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

    pub fn update_faculties(&mut self, new_faculties: &Vec<Faculty>) -> DBResult<()> {
        update_table!(
            &mut self.conn,
            schema::faculties::dsl::faculties,
            new_faculties
        )
    }

    /// Returns all current faculties of the RUDN university
    /// If the vector is empty, something is wrong with the database
    pub fn get_faculties(&mut self) -> DBResult<Vec<Faculty>> {
        use schema::faculties::dsl::*;
        faculties.load::<Faculty>(&mut self.conn).map_err(|e| {
            log::error!("Error: '{e}' while retrieving faculties from the database");
            DBError::RetrieveError(String::from(
                "Could not retreive faculties from the database",
            ))
        })
    }

    pub fn update_groups(&mut self, new_groups: &Vec<Group>) -> DBResult<()> {
        update_table!(&mut self.conn, schema::groups::dsl::groups, new_groups)
    }

    pub fn get_groups_for_faculty(
        &mut self,
        faculty_uuid: &Uuid,
    ) -> DBResult<HashMap<Uuid, Vec<Group>>> {
        use schema::groups::dsl::*;
        get_filtered_table_vec_data!(
            &mut self.conn,
            groups,
            Group,
            (Uuid, faculty),
            [(faculty, faculty_uuid)]
        )
    }

    pub fn get_timetable_for_group(&mut self, group: &Uuid) -> DBResult<HashMap<Day, Vec<Event>>> {
        use schema::timetables::dsl::*;
        get_filtered_table_vec_data!(
            &mut self.conn,
            timetables,
            Event,
            (Day, day),
            [(student_group, group)]
        )
    }

    pub fn update_timetable(&mut self, timetable: &HashMap<Day, Vec<Event>>) -> DBResult<()> {
        update_table!(
            &mut self.conn,
            schema::timetables::dsl::timetables,
            timetable
                .iter()
                .flat_map(|(_, v)| v.clone())
                .map(InsertableEvent::from)
                .collect::<Vec<_>>()
        )
    }
}
