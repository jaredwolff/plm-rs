#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
pub mod models;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use self::models::NewPart;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_part(conn: &SqliteConnection, pn: &str, mpn: &str, descr: &str, ver: &i32) -> usize {
    use schema::parts;

    let new_part = NewPart { pn, mpn, descr, ver };

    // TODO: match the error for create_part
    diesel::insert_into(parts::table)
        .values(&new_part)
        .execute(conn)
        .expect("Error saving new part")
}