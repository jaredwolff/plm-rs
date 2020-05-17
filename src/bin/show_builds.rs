extern crate diesel;
extern crate mrp;

#[macro_use]
extern crate prettytable;
use prettytable::Table;

use self::diesel::prelude::*;
use self::models::*;
use self::mrp::*;

fn main() {
  use mrp::schema::builds::dsl::*;

  // Create the table
  let mut table = Table::new();

  let connection = establish_connection();
  let results = builds
    .load::<Build>(&connection)
    .expect("Error loading builds");

  println!("Displaying {} builds", results.len());
  table.add_row(row![
    "Build ID", "PN", "Ver", "Notes", "Complete", "Quantity"
  ]);
  for build in results {
    // Get the part info..
    let part = find_part_by_id(&connection, &build.part_id).expect("Unable to get build part.");
    table.add_row(row![
      build.id,
      part.pn,
      build.part_ver,
      build.notes.unwrap(),
      build.complete,
      build.quantity
    ]);
  }
  table.printstd();
}
