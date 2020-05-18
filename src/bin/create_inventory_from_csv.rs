// Read CSV
// Map columns to match inventory

// pub struct Inventory {
//   pub id: i32,
//   pub quantity: i32,
//   pub unit_price: Option<f32>,
//   pub notes: Option<String>,
//   pub created_at: NaiveDateTime,
//   pub part_id: i32,
// }

// Function then creates new inventory records for each line. This adds to the existing inventory (i.e. it does not reset it)
// This is for adding orders from Digikey and the like

extern crate diesel;
extern crate mrp;

use self::mrp::*;
use models::*;

use std::env::args;
use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
  part_number: String,
  quantity: i32,
  notes: String,
  unit_price: f32,
}

fn main() {
  // Establish connection!
  let conn = establish_connection();

  // Takes a .sch file as an input
  let filename = args().nth(1).expect("Need a filename as an argument.");

  // Open the file
  let file = File::open(filename).unwrap();
  let file = BufReader::new(file);

  let mut records: Vec<Record> = Vec::new();

  let mut rdr = csv::Reader::from_reader(file);
  for result in rdr.deserialize() {
    // Notice that we need to provide a type hint for automatic
    // deserialization.
    let record: Record = result.expect("Unable to deserialize.");
    // println!("{:?}", record);
    records.push(record);
  }

  for record in &records {
    // Check if part number exists
    let part = find_part_by_pn(&conn, &record.part_number);

    // If theres an error exit so the user can fix the problem.
    if part.is_err() {
      println!(
        "{} was not found! No changes were made.",
        record.part_number
      );
      std::process::exit(1);
    }
  }

  // Re iterate now that we know the parts are all valid
  for record in &records {
    // Check if part number exists
    let part = find_part_by_pn(&conn, &record.part_number).expect("Unable to get part.");

    // Commits change
    let entry = NewInventoryEntry {
      part_id: &part.id,
      unit_price: Some(&record.unit_price),
      quantity: &record.quantity,
      notes: Some(&record.notes),
    };

    // Finally create the inventory if all look ok!
    create_inventory(&conn, &entry).expect("Unable to create inventory item.");

    // Print out that it was successful
    println!("Created inventory for {}!", part.pn);
  }
}
