// Completing a build will:
// Double check inventory
// Then once permission is granted:
// Calculate unit price (add up all inventory and divide by build count)
// Add an inventory entry to mark used.

extern crate diesel;
extern crate mrp;

use self::diesel::prelude::*;
use self::models::*;
use self::mrp::*;

struct Shortage {
  pid: i32,
  pn: String,
  refdes: String,
  needed: i32,
  short: i32,
}

use std::env::args;
use std::io;

fn main() {
  use mrp::schema::*;

  // Establish connection!
  let conn = establish_connection();

  // Takes a .sch file as an input
  let build_id = args().nth(1).expect("Need a build number as an argument.");
  let build_id = build_id.parse::<i32>().unwrap();

  // Get the build
  let build = find_build_by_id(&conn, &build_id).expect("Unable to find build!");

  // Get partslist
  let bom_list = parts_parts::dsl::parts_parts
    .filter(parts_parts::dsl::bom_part_id.eq(build.part_id))
    .filter(parts_parts::dsl::bom_ver.eq(build.part_ver))
    .load::<PartsPart>(&conn)
    .expect("Error loading parts");

  let mut shortages: Vec<Shortage> = Vec::new();

  // Iterate though the results and check inventory
  for bom_list_entry in &bom_list {
    // Skip if nostuff is set
    if bom_list_entry.nostuff == 1 {
      println!("{} is no stuff.", bom_list_entry.refdes);
      continue;
    }

    // Serach for part in inventory. Do calculations as necessary.
    let mut quantity = 0;

    let inventory_entries = find_inventories_by_part_id(&conn, &bom_list_entry.part_id)
      .expect("Unable to query for inventory");

    // Calculate the quantity
    for entry in inventory_entries {
      quantity += entry.quantity;
    }

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
        find_part_by_id(&conn, &bom_list_entry.part_id).expect("Unable to get part by id.");

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
        refdes: bom_list_entry.refdes.clone(),
        needed: build.quantity * bom_list_entry.quantity,
        short: short,
      };

      // Add to shortage list
      shortages.push(shortage);
    }
  }

  let mut still_short = false;

  // Make sure that all parts are not short.
  for entry in shortages {
    if entry.short != 0 {
      println!(
        "Still short {} for part: {} ({})",
        entry.needed, entry.pn, entry.refdes
      );
      if still_short == false {
        still_short = true;
      }
    }
  }

  // For prompts
  let stdio = io::stdin();
  let input = stdio.lock();
  let output = io::stdout();

  let mut prompt = prompt::Prompt {
    reader: input,
    writer: output,
  };

  // Return
  if still_short {
    std::process::exit(1);
  }

  let resp = prompt.ask_yes_no_question("Would you like to finish the build?");

  if resp {
    // "Finish" the build

    // Used to calculate total cost
    let mut total_cost = 0.0;

    // Iterate though every bom list entry
    // Do the math to modify the inventory
    for bom_list_entry in &bom_list {
      // Skip if nostuff is set
      if bom_list_entry.nostuff == 1 {
        println!("{} is no stuff.", bom_list_entry.refdes);
        continue;
      }

      // Track the quantity
      let mut quantity = bom_list_entry.quantity;

      // Inventory entries
      let inventory_entries = find_inventories_by_part_id(&conn, &bom_list_entry.part_id)
        .expect("Unable to query for inventory");

      // Calculate the quantity
      for entry in inventory_entries {
        let mut new_qty;
        let mut used;

        // Calculate quantities
        if entry.quantity >= quantity {
          new_qty = entry.quantity - quantity;
          used = quantity;
          quantity = 0;
        } else {
          new_qty = 0;
          used = entry.quantity;
          quantity = quantity - entry.quantity;
        }

        // Get string from entry.notes
        let notes = match entry.notes {
          Some(x) => x,
          None => "".to_string(),
        };

        // Create update
        let update = NewUpdateInventoryEntry {
          quantity: &new_qty,
          consumed: &used,
          unit_price: entry.unit_price.as_ref(),
          notes: Some(&notes),
          part_ver: &entry.part_ver,
          part_id: &entry.part_id,
        };

        // Push this inventory item
        update_inventory_by_id(&conn, &entry.id, &update).expect("Unable to create inventory.");

        // Add the cost used to total_cost
        if entry.unit_price.is_some() {
          total_cost += used as f32 * entry.unit_price.unwrap();
        }

        // Break once we get the necessary quantity
        if quantity == 0 {
          break;
        }
      }

      // Repeat until complete!
    }

    // Calculate unit cost
    let unit_cost = total_cost / build.quantity as f32;
    println!("Total cost: ${}(USD)", total_cost);
    println!("Unit cost: ${}(USD)", unit_cost);

    let build_name = format!("Build {}", build_id);

    // Create inventory of assemblies built
    let new_inventory = NewUpdateInventoryEntry {
      quantity: &build.quantity,
      consumed: &0,
      unit_price: Some(&unit_cost),
      notes: Some(&build_name),
      part_ver: &build.part_ver,
      part_id: &build.part_id,
    };

    // Get string from entry.notes
    let notes = match build.notes {
      Some(x) => x,
      None => "".to_string(),
    };

    // Udate build complete
    let update_build = NewUpdateBuild {
      quantity: &build.quantity,
      complete: &1,
      notes: Some(&notes),
      part_ver: &build.part_ver,
      part_id: &build.part_id,
    };

    // Update build by id
    update_build_by_id(&conn, &build.id, &update_build).expect("Unable to update build!");

    // Push this inventory item
    create_inventory(&conn, &new_inventory).expect("Unable to create inventory.");
  }
}
