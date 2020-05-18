extern crate diesel;
extern crate mrp;

use prettytable::Table;

use self::diesel::prelude::*;
use self::models::*;
use self::mrp::*;

use std::fs::File;
use std::io::BufReader;

use std::io;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
  part_number: String,
  quantity: i32,
  notes: String,
  unit_price: f32,
}

#[derive(Debug)]
struct Shortage {
  pid: i32,
  pn: String,
  desc: String,
  have: i32,
  needed: i32,
  short: i32,
}

pub fn create_from_file(filename: &String) {
  // Establish connection!
  let conn = establish_connection();

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
    let entry = NewUpdateInventoryEntry {
      part_id: &part.id,
      part_ver: &part.ver,
      unit_price: Some(&record.unit_price),
      quantity: &record.quantity,
      consumed: &0,
      notes: Some(&record.notes),
    };

    // Finally create the inventory if all look ok!
    create_inventory(&conn, &entry).expect("Unable to create inventory item.");

    // Print out that it was successful
    println!("Created inventory for {}!", part.pn);
  }
}

pub fn create() {
  // For prompts
  let stdio = io::stdin();
  let input = stdio.lock();
  let output = io::stdout();

  let mut prompt = prompt::Prompt {
    reader: input,
    writer: output,
  };

  // Prompts for a part number
  let part_number = prompt.ask_text_entry("Enter part number:");

  // Establish connection!
  let conn = establish_connection();

  // Check if part number exists
  let part = find_part_by_pn(&conn, &part_number);

  // Make sure we're valid!
  let part = match part {
    Ok(x) => x,
    Err(_) => {
      // println!("Unable to find {}", part_number);
      std::process::exit(1);
    }
  };

  // Then an ajustment value
  let adj = prompt.ask_text_entry("Enter adjustment value:");
  let adj: i32 = adj.trim().parse().expect("Invalid adjustment!");

  // Then any notes.
  let notes = prompt.ask_text_entry("Enter notes:");

  println!("Part number: {}", part.pn);
  println!("Ajustment: {}", adj);
  // println!("Notes: {}", notes);
  let proceed = prompt.ask_yes_no_question("Look ok?");

  // Confirm change (y/n)
  if proceed {
    // Commits change
    let entry = NewUpdateInventoryEntry {
      part_id: &part.id,
      part_ver: &part.ver,
      unit_price: Some(&0.0),
      quantity: &adj,
      consumed: &0,
      notes: Some(&notes),
    };

    create_inventory(&conn, &entry).expect("Unable to create inventory item.");
  }
}

pub fn show() {
  use mrp::schema::inventories::dsl::*;

  // Create the table
  let mut table = Table::new();

  let connection = establish_connection();
  let results = inventories
    .load::<Inventory>(&connection)
    .expect("Error loading parts");

  println!("Displaying {} parts", results.len());
  table.add_row(row![
    "PN",
    "Desc",
    "Qty",
    "Consumed",
    "Unit Price",
    "Notes",
    "Ver"
  ]);
  for inventory in results {
    // Check if part number exists
    let part = find_part_by_id(&connection, &inventory.part_id).expect("Unable to get part.");

    table.add_row(row![
      part.pn,
      part.descr,
      inventory.quantity,
      inventory.consumed,
      inventory.unit_price.unwrap_or(0.0),
      inventory.notes.unwrap_or("".to_string()),
      inventory.part_ver
    ]);
  }
  table.printstd();
}

pub fn show_shortage() {
  use mrp::schema::*;

  // Create the table
  let mut table = Table::new();

  let connection = establish_connection();
  let results = builds::dsl::builds
    .load::<Build>(&connection)
    .expect("Error loading builds");

  let mut shortages: Vec<Shortage> = Vec::new();

  // Iterate though the builds,
  // Create a table of all parts and computed inventory
  // and shortages (indicated in - or + numbers)
  for build in results {
    // Skip over to the next one. This build is done!
    if build.complete == 1 {
      continue;
    }

    // First get the parts.
    let bom_list = parts_parts::dsl::parts_parts
      .filter(parts_parts::dsl::bom_part_id.eq(build.part_id))
      .filter(parts_parts::dsl::bom_ver.eq(build.part_ver))
      .load::<PartsPart>(&connection)
      .expect("Error loading parts");

    // Iterate though the results and check inventory
    for bom_list_entry in bom_list {
      // Skip if nostuff is set
      if bom_list_entry.nostuff == 1 {
        println!("item is no stuff {}", bom_list_entry.refdes);
        continue;
      }

      // Serach for part in inventory. Do calculations as necessary.
      let mut quantity = 0;

      let inventory_entries = find_inventories_by_part_id(&connection, &bom_list_entry.part_id)
        .expect("Unable to query for inventory");

      // Calculate the quantity
      for entry in inventory_entries {
        quantity += entry.quantity;
      }

      // TODO: push a new temp inventory struct
      // This struct has, inventory quantity (+/-), quantity needed, part name
      let mut found_in_shortage_list = false;

      // Check in shortage list, do some calculations if that item exists
      for mut entry in &mut shortages {
        if entry.pid == bom_list_entry.part_id {
          // Set short to 0 if > 0
          let mut short = quantity - entry.needed;
          if short > 0 {
            short = 0;
          }

          // Then set the variables
          entry.needed += build.quantity * bom_list_entry.quantity;
          entry.short = short;
          found_in_shortage_list = true;
          break;
        }
      }

      if !found_in_shortage_list {
        // Get the part for more info
        let part =
          find_part_by_id(&connection, &bom_list_entry.part_id).expect("Unable to get part by id.");

        // Calculate the amount short
        let mut short = quantity - (build.quantity * bom_list_entry.quantity);

        // To 0 if not short
        if short > 0 {
          short = 0;
        }

        // Create shortage item
        let shortage = Shortage {
          pid: bom_list_entry.part_id,
          pn: part.pn,
          desc: part.descr,
          have: quantity,
          needed: build.quantity * bom_list_entry.quantity,
          short: short,
        };

        // Add to shortage list
        shortages.push(shortage);
      }
    }
  }

  // Print out the shortages in table format.
  table.add_row(row!["PID", "PN", "Desc", "Have", "Needed", "Short",]);

  for entry in shortages {
    table.add_row(row![
      entry.pid,
      entry.pn,
      entry.desc,
      entry.have,
      entry.needed,
      entry.short,
    ]);
  }

  table.printstd();
}
