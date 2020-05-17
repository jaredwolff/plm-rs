extern crate diesel;
extern crate mrp;

#[macro_use]
extern crate prettytable;
use prettytable::Table;

use self::diesel::prelude::*;
use self::models::*;
use self::mrp::*;

use std::env::args;

fn main() {
  use mrp::schema::*;

  // Establish connection
  let connection = establish_connection();

  // Takes a .sch file as an input
  let part_number = args().nth(1).expect("Need a part number as an argument.");
  let version = args()
    .nth(2)
    .expect("Need a version number as an argument.");
  let version = version.parse::<i32>().unwrap();

  let parts = find_parts_by_pn(&connection, &part_number).expect("Unable to run query.");

  if parts.len() == 0 {
    println!("{} version {} was not found!", part_number, version);
    std::process::exit(1);
  }

  // Transform the response into a Part
  let part = &parts[0];

  if part.ver != version {
    println!(
      "{} version {} was not found! Latest is: {}",
      part_number, version, part.ver
    );
    std::process::exit(1);
  }

  // Create the table
  let mut table = Table::new();

  let results = parts_parts::dsl::parts_parts
    .filter(parts_parts::dsl::bom_part_id.eq(part.id))
    .filter(parts_parts::dsl::bom_ver.eq(version))
    .load::<PartsPart>(&connection)
    .expect("Error loading parts");

  println!("Displaying {} parts", results.len());

  println!(
    "Part Number: {} BOM Id: {} Version: {}",
    part.pn, part.id, part.ver
  );

  table.add_row(row![
    "Quantity",
    "Refdes",
    "PN",
    "MPN",
    "Desc",
    "Ver",
    "Inventory Qty"
  ]);
  for entry in results {
    let details =
      find_part_by_id(&connection, &entry.part_id.unwrap()).expect("Unable to get details!");

    // Get inventory info
    let inventory = inventories::dsl::inventories
      .filter(inventories::dsl::part_id.eq(entry.part_id.unwrap()))
      .load::<Inventory>(&connection)
      .expect("Error loading parts");

    let mut inventory_qty = 0;

    // Tally inventory
    for item in inventory {
      // Increment quantity
      inventory_qty += item.quantity;
    }

    table.add_row(row![
      entry.quantity,
      entry.refdes,
      details.pn,
      details.mpn,
      details.descr,
      details.ver,
      inventory_qty
    ]);
  }
  table.printstd();
}
