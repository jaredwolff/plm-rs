#[macro_use]
extern crate diesel;
extern crate dotenv;

#[macro_use]
extern crate diesel_migrations;

pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use self::models::*;

// Migrate
embed_migrations!();

pub fn establish_connection() -> SqliteConnection {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  SqliteConnection::establish(&database_url)
    .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

// Part related
pub fn create_part(
  conn: &SqliteConnection,
  part: &NewUpdatePart,
) -> std::result::Result<usize, diesel::result::Error> {
  use schema::parts;

  diesel::insert_into(parts::table).values(part).execute(conn)
}

pub fn update_part(
  conn: &SqliteConnection,
  part: &NewUpdatePart,
) -> std::result::Result<usize, diesel::result::Error> {
  use schema::parts;

  // TODO: provides part ID?
  // TODO: match the error for create_part
  diesel::update(parts::dsl::parts.filter(parts::dsl::pn.eq(part.pn)))
    .set(part)
    .execute(conn)
}

pub fn find_part_by_pn(
  conn: &SqliteConnection,
  pn: &str,
) -> std::result::Result<Part, diesel::result::Error> {
  use schema::parts;

  parts::dsl::parts.filter(parts::dsl::pn.eq(pn)).first(conn)
}

pub fn find_part_by_pn_and_ver(
  conn: &SqliteConnection,
  pn: &str,
  ver: &i32,
) -> std::result::Result<Part, diesel::result::Error> {
  use schema::parts;

  parts::dsl::parts
    .filter(parts::dsl::pn.eq(pn))
    .filter(parts::dsl::ver.eq(ver))
    .first(conn)
}

pub fn find_part_by_id(
  conn: &SqliteConnection,
  id: &i32,
) -> std::result::Result<Part, diesel::result::Error> {
  use schema::parts;

  parts::dsl::parts.filter(parts::dsl::id.eq(id)).first(conn)
}

pub fn create_bom_line_item(
  conn: &SqliteConnection,
  part: &NewPartsParts,
) -> std::result::Result<usize, diesel::result::Error> {
  use schema::parts_parts;

  diesel::insert_into(parts_parts::table)
    .values(part)
    .execute(conn)
}

// Build related

pub fn create_build(
  conn: &SqliteConnection,
  build: &NewBuild,
) -> std::result::Result<usize, diesel::result::Error> {
  use schema::builds;

  diesel::insert_into(builds::table)
    .values(build)
    .execute(conn)
}

pub fn find_builds_by_pn(
  conn: &SqliteConnection,
  pn: &str,
) -> std::result::Result<Vec<Build>, diesel::result::Error> {
  use schema::builds;

  let part = find_part_by_pn(&conn, &pn).expect("Unable to run parts query.");

  builds::dsl::builds
    .filter(builds::dsl::part_id.eq(part.id))
    .load::<Build>(conn)
}

// Inventory related

pub fn create_inventory(
  conn: &SqliteConnection,
  entry: &NewInventoryEntry,
) -> std::result::Result<usize, diesel::result::Error> {
  use schema::inventories;

  diesel::insert_into(inventories::table)
    .values(entry)
    .execute(conn)
}

pub fn find_inventories_by_part_id(
  conn: &SqliteConnection,
  id: &i32,
) -> std::result::Result<Vec<Inventory>, diesel::result::Error> {
  use schema::inventories;

  inventories::dsl::inventories
    .filter(inventories::dsl::part_id.eq(id))
    .load::<Inventory>(conn)
}

#[allow(dead_code)]
pub fn test_connection() -> SqliteConnection {
  // Start a connection from memory
  let conn = SqliteConnection::establish(":memory:").expect("Unable to establish db in memory!");

  // This will run the necessary migrations.
  embedded_migrations::run(&conn).expect("Unable to run test migration.");

  // Return the active connection
  conn
}

/* START: Part Related Tests */
mod part_tests {

  #[test]
  fn create_part_check_if_created() {
    use super::*;
    use models::Part;
    use schema::parts::dsl::*;

    let conn = test_connection();

    // Create NewUpdatePart instance
    let part = NewUpdatePart {
      pn: "CAP-0.1U-10V-0402",
      mpn: "ABCD",
      descr: "CAP 0.1U 10V 0402",
      ver: &1,
    };

    // Create the part
    create_part(&conn, &part).expect("Error creating part!");

    // Serach for it and make sure that it matches
    let found: Part = parts.find(1).first(&conn).unwrap();

    // Make sure these guys are equal
    assert_eq!(part.pn, found.pn);
    assert_eq!(part.mpn, found.mpn);
    assert_eq!(part.descr, found.descr);
    assert_eq!(*part.ver, found.ver);
  }

  #[test]
  #[should_panic]
  // This is testing the schema more than anything
  // Only one part with the same PN!
  fn create_duplicate_pn_should_panic() {
    use super::*;
    let conn = test_connection();

    // Create NewUpdatePart instance
    let part = NewUpdatePart {
      pn: "CAP-0.1U-10V-0402",
      mpn: "ABCD",
      descr: "CAP 0.1U 10V 0402",
      ver: &1,
    };

    // Create the part
    create_part(&conn, &part).expect("Error creating part!");

    // Create NewUpdatePart instance
    let part = NewUpdatePart {
      pn: "CAP-0.1U-10V-0402",
      mpn: "ABCD-ND",
      descr: "CAP 0.1U 10V 0402",
      ver: &1,
    };

    // Do it again
    create_part(&conn, &part).expect("Error creating part!");
  }

  #[test]
  #[should_panic]
  // This is testing the schema more than anything
  // Only one part with the same MPN!
  fn create_duplicate_mpn_should_panic() {
    use super::*;
    let conn = test_connection();

    // Create NewUpdatePart instance
    let part = NewUpdatePart {
      pn: "CAP-0.1U-10V-0402",
      mpn: "ABCD",
      descr: "CAP 0.1U 10V 0402",
      ver: &1,
    };

    // Create the part
    create_part(&conn, &part).expect("Error creating part!");

    // Create NewUpdatePart instance
    let part = NewUpdatePart {
      pn: "CAP-0.1U-10V-0402-01",
      mpn: "ABCD",
      descr: "CAP 0.1U 10V 0402",
      ver: &1,
    };

    // Do it again
    create_part(&conn, &part).expect("Error creating part!");
  }

  #[test]
  fn create_and_update_part() {
    use super::*;
    use models::Part;
    use schema::parts::dsl::*;

    let conn = test_connection();

    // Create NewUpdatePart instance
    let part = NewUpdatePart {
      pn: "CAP-0.1U-10V-0402",
      mpn: "ABCD",
      descr: "CAP 0.1U 10V 0402",
      ver: &1,
    };

    // Create the part
    create_part(&conn, &part).expect("Error creating part!");

    // Update the value
    let part = NewUpdatePart {
      pn: "CAP-0.1U-10V-0402",
      mpn: "ABCD",
      descr: "CAP 0.1 10V 0402 GOOD", // Only changing this guy
      ver: &1,
    };

    // Update the part
    update_part(&conn, &part).expect("Error creating part!");

    // Serach for it and make sure that it matches
    let found: Part = parts.find(1).first(&conn).unwrap();

    // Make sure these guys are equal
    assert_eq!(part.descr, found.descr);
  }
}

/* START: Inventory Related Tests */
mod inventory_tests {}

/* START: Build Related Tests */
mod build_tests {}
