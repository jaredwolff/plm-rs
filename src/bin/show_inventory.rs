extern crate diesel;
extern crate mrp;

#[macro_use]
extern crate prettytable;
use prettytable::Table;

use self::diesel::prelude::*;
use self::models::*;
use self::mrp::*;

fn main() {
  use mrp::schema::inventories::dsl::*;

  // Create the table
  let mut table = Table::new();

  let connection = establish_connection();
  let results = inventories
    .load::<Inventory>(&connection)
    .expect("Error loading parts");

  println!("Displaying {} parts", results.len());
  table.add_row(row!["PN", "Desc", "Qty", "Unit Price", "Notes"]);
  for inventory in results {
    // Check if part number exists
    let part = find_part_by_id(&connection, &inventory.part_id).expect("Unable to get part.");

    table.add_row(row![
      part.pn,
      part.descr,
      inventory.quantity,
      inventory.unit_price.unwrap_or(0.0),
      inventory.notes.unwrap_or("".to_string())
    ]);
  }
  table.printstd();
}
