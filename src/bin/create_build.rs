extern crate diesel;
extern crate mrp;

use self::models::*;
use self::mrp::*;

use std::env::args;

fn main() {
  // Establish connection
  let connection = establish_connection();

  // Takes a .sch file as an input
  let part_number = args().nth(1).expect("Need a part number as an argument.");
  let version = args()
    .nth(2)
    .expect("Need a version number as an argument.");
  let version = version.parse::<i32>().unwrap();
  let quantity = args().nth(3).expect("Need a quantity as an argument.");
  let quantity = quantity.parse::<i32>().unwrap();

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

  let build = NewBuild {
    quantity: &quantity,
    complete: &0,
    notes: "",
    part_ver: &version,
    part_id: &part.id,
  };

  create_build(&connection, &build).expect("Unable to create build!");

  println!(
    "Created build of {} ver: {} with qty: {}",
    part.pn, part.ver, quantity
  );
}
