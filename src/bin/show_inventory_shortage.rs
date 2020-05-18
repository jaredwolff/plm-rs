extern crate diesel;
extern crate mrp;

#[macro_use]
extern crate prettytable;
use prettytable::Table;

use self::diesel::prelude::*;
use self::models::*;
use self::mrp::*;

struct Shortage {
  pid: i32,
  pn: String,
  desc: String,
  have: i32,
  needed: i32,
  short: i32,
}

fn main() {
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
