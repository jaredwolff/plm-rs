extern crate diesel;

use crate::{models::*, *};
use prettytable::Table;

use anyhow::anyhow;

use self::diesel::prelude::*;

use std::io::{BufReader, BufWriter};
use std::{fmt::Debug, fs::File};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct NewInventoryRecord {
    mpn: String,
    quantity: Option<i32>,
    notes: Option<String>,
    unit_price: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InventoryEntry {
    pub id: i32,
    pub mpn: String,
    pub quantity: i32,
    pub consumed: i32,
    pub unit_price: Option<f32>,
    pub notes: Option<String>,
    pub part_ver: i32,
    pub part_id: i32,
}

#[derive(Debug, Serialize)]
pub struct Shortage {
    pub pid: i32,
    pub pn: String,
    pub mpn: String,
    pub desc: String,
    pub have: i32,
    pub needed: i32,
    pub short: i32,
    pub quantity: Option<i32>,
    pub notes: Option<String>,
    pub unit_price: Option<f32>,
}

/// Reads records from file using a generic type. Useful across create and update calls
fn read_records<T>(filename: &str) -> anyhow::Result<Vec<T>>
where
    T: DeserializeOwned + Debug,
{
    // Open the file
    let file = File::open(filename).unwrap();
    let file = BufReader::new(file);

    let mut records: Vec<T> = Vec::new();

    let mut rdr = csv::Reader::from_reader(file);

    // Process each line entry.
    for (pos, result) in rdr.deserialize().enumerate() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: T = match result {
            Ok(r) => r,
            Err(e) => return Err(anyhow!("Unable to process line {}. Error: {}", pos, e)),
        };

        println!("Processing: {:?}", record);
        records.push(record);
    }

    Ok(records)
}

// Update from inventory export file
pub fn update_from_file(app: &mut crate::Application, filename: &str) {
    // Get records from file
    let records: Vec<InventoryEntry> = match read_records(filename) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}\nNo changes have been made", e);
            return;
        }
    };

    // Only updates records found!
    for record in &records {
        // Notes converted as necessary
        let notes = record.notes.as_deref();

        // Convert from InventoryRecord to NewUpdateInventoryEntry
        let update = NewUpdateInventoryEntry {
            quantity: &record.quantity,
            consumed: &record.consumed,
            unit_price: record.unit_price.as_ref(),
            notes,
            part_ver: &record.part_ver,
            part_id: &record.part_id,
        };

        // Then update the entry as needed
        if let Err(e) = update_inventory_by_id(&app.conn, &record.id, &update) {
            eprintln!("Error updating inventory id: {}. Error: {}", record.id, e);
        } else {
            println!("Updated: {}", record.mpn);
        }
    }
}

pub fn create_from_file(app: &mut crate::Application, filename: &str) {
    println!("{:?}", app.config);
    println!("{:?}", filename);

    // Get records from file
    let records: Vec<NewInventoryRecord> = match read_records(filename) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}\nNo changes have been made", e);
            return;
        }
    };

    for record in &records {
        println!("Finding: \"{}\"", record.mpn);

        // Check if part number exists
        // Uses MPN as it's the common denominator between this and Digikey/Arrow/Mouser etc.
        let part = find_part_by_mpn(&app.conn, &record.mpn);

        // If theres an error exit so the user can fix the problem.
        match part {
            Err(e) => {
                println!(
                    "{} was not found! No changes were made. Error: {}",
                    record.mpn, e
                );
                std::process::exit(1);
            }
            _ => {
                continue;
            }
        }
    }

    // Re iterate now that we know the parts are all valid
    for record in &records {
        // We need at least a quantity to add a new record
        let quantity = match record.quantity {
            Some(q) => q,
            None => continue,
        };

        // Get the notes
        let notes = record.notes.as_deref();

        // Check if part number exists
        let part = find_part_by_mpn(&app.conn, &record.mpn).expect("Unable to get part.");

        // Commits change
        let entry = NewUpdateInventoryEntry {
            part_id: &part.id,
            part_ver: &part.ver,
            unit_price: record.unit_price.as_ref(),
            quantity: &quantity,
            consumed: &0,
            notes,
        };

        // Finally create the inventory if all look ok!
        create_inventory(&app.conn, &entry).expect("Unable to create inventory item.");

        // Print out that it was successful
        println!("Created inventory for {}!", part.pn);
    }
}

pub fn create(app: &mut crate::Application) {
    // app.prompts for a part number
    let part_number = app.prompt.ask_text_entry("Enter part number: ");

    // Check if part number exists
    let part = find_part_by_pn(&app.conn, &part_number);

    // Make sure we're valid!
    let part = match part {
        Ok(x) => x,
        Err(_) => {
            println!("Unable to find {}", part_number);
            std::process::exit(1);
        }
    };

    // Then an ajustment value
    let adj = app.prompt.ask_text_entry("Enter adjustment value: ");
    let adj: i32 = adj.trim().parse().expect("Invalid adjustment!");

    // Unit price
    let price = app.prompt.ask_text_entry("Enter unit price: ");
    let price: f32 = price.trim().parse().expect("Invalid price!");

    // Then any notes.
    let notes = app.prompt.ask_text_entry("Enter notes: ");

    println!("Part number: {}", part.pn);
    println!("Ajustment: {}", adj);
    println!("Price: ${}", price);
    println!("Notes: {}", notes);
    let proceed = app.prompt.ask_yes_no_question("Look ok?");

    // Confirm change (y/n)
    if proceed {
        // Commits change
        let entry = NewUpdateInventoryEntry {
            part_id: &part.id,
            part_ver: &part.ver,
            unit_price: Some(&price),
            quantity: &adj,
            consumed: &0,
            notes: Some(&notes),
        };

        create_inventory(&app.conn, &entry).expect("Unable to create inventory item.");
    }
}

pub fn show(app: &mut crate::Application, show_all_entries: bool) {
    use crate::schema::inventories::dsl::*;

    // Create the table
    let mut table = Table::new();

    let results = inventories
        .load::<Inventory>(&app.conn)
        .expect("Error loading parts");

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
        // Check if show_all_entries
        if !show_all_entries && inventory.quantity == 0 {
            continue;
        }

        // Check if part number exists
        let part = find_part_by_id(&app.conn, &inventory.part_id).expect("Unable to get part.");

        table.add_row(row![
            part.pn,
            part.descr,
            inventory.quantity,
            inventory.consumed,
            inventory.unit_price.unwrap_or(0.0),
            inventory.notes.unwrap_or_else(|| "".to_string()),
            inventory.part_ver
        ]);
    }

    // Change output depending on how many parts (or lack thereof)
    if table.len() == 1 {
        println!("No inventory to display.");
    } else {
        println!("Displaying {} parts", table.len() - 1);
        table.printstd();
    }
}

// TODO: show shortage by build ID
// Defualt hide non-short items. Option to view all.
pub fn show_shortage(app: &mut crate::Application, show_all_entries: bool) {
    // Create the table
    let mut table = Table::new();

    // Print out the shortages in table format.
    table.add_row(row!["PID", "PN", "MPN", "Desc", "Have", "Needed", "Short",]);

    let shortages = get_shortages(app, show_all_entries);

    let shortages = match shortages {
        Ok(x) => x,
        Err(e) => {
            println!("Error getting shortages: {:?}", e);
            std::process::exit(1);
        }
    };

    for entry in shortages {
        table.add_row(row![
            entry.pid,
            entry.pn,
            entry.mpn,
            entry.desc,
            entry.have,
            entry.needed,
            entry.short,
        ]);
    }

    table.printstd();
}

// Export inventory to csv
pub fn export_to_file(app: &mut crate::Application, filename: &str, export_all: bool) {
    use crate::schema::*;

    // Run the query
    let inventory = inventories::dsl::inventories
        .load::<Inventory>(&app.conn)
        .expect("Uanble to load inventory list.");

    // File operations
    let file = File::create(filename).unwrap();
    let file = BufWriter::new(file);

    // Create CSV writer
    let mut wtr = csv::Writer::from_writer(file);

    // Iterate and add to csv
    for entry in inventory {
        // Skips this part if qty = 0 if export_all is false
        if !export_all && entry.quantity == 0 {
            continue;
        }

        // Grabs the part information
        let part = find_part_by_id(&app.conn, &entry.part_id).unwrap();

        // Create a new entry
        let inventory_entry = InventoryEntry {
            id: entry.id,
            mpn: part.mpn,
            quantity: entry.quantity,
            consumed: entry.consumed,
            unit_price: entry.unit_price,
            notes: entry.notes,
            part_ver: entry.part_ver,
            part_id: entry.part_id,
        };

        wtr.serialize(inventory_entry)
            .expect("Unable to serialize.");
        wtr.flush().expect("Unable to flush");
    }

    println!("Inventory list exported to {}", filename);
}

// Export shortages to csv
pub fn export_shortages_to_file(app: &mut crate::Application, filename: &str) {
    let shortages = get_shortages(app, false).expect("Unable to get shortage report.");

    let file = File::create(filename).unwrap();
    let file = BufWriter::new(file);

    // Create CSV writer
    let mut wtr = csv::Writer::from_writer(file);

    // Iterate and add to csv
    for shortage in shortages {
        wtr.serialize(shortage).expect("Unable to serialize.");
        wtr.flush().expect("Unable to flush");
    }

    println!("Shortages exported to {}", filename);
}

pub fn get_shortages(
    app: &mut crate::Application,
    show_all_entries: bool,
) -> std::result::Result<Vec<Shortage>, diesel::result::Error> {
    use crate::schema::*;

    let results = builds::dsl::builds
        .filter(builds::dsl::complete.eq(0)) // Only show un-finished builds
        .load::<Build>(&app.conn);

    // Return the error if there was an issue
    let results = match results {
        Ok(x) => x,
        Err(e) => return Err(e),
    };

    let mut shortages: Vec<Shortage> = Vec::new();

    // Iterate though the builds,
    // Create a table of all parts and computed inventory
    // and shortages (indicated in - or + numbers)
    for build in results {
        // First get the parts.
        let bom_list = parts_parts::dsl::parts_parts
            .filter(parts_parts::dsl::bom_part_id.eq(build.part_id))
            .filter(parts_parts::dsl::bom_ver.eq(build.part_ver))
            .load::<PartsPart>(&app.conn);

        // Return the error if there was an issue
        let bom_list = match bom_list {
            Ok(x) => x,
            Err(e) => return Err(e),
        };

        // Iterate though the results and check inventory
        for bom_list_entry in bom_list {
            // Skip if nostuff is set
            if bom_list_entry.nostuff == 1 {
                continue;
            }

            // Serach for part in inventory. Do calculations as necessary.
            let mut inventory_quantity = 0;

            // Get the inventory entries
            let inventory_entries = find_inventories_by_part_id(&app.conn, &bom_list_entry.part_id);

            // Return the error if there was an issue
            let inventory_entries = match inventory_entries {
                Ok(x) => x,
                Err(e) => return Err(e),
            };

            // Calculate the quantity
            for entry in inventory_entries {
                inventory_quantity += entry.quantity;
            }

            // This struct has, inventory quantity (+/-), quantity needed, part name
            let mut found_in_shortage_list = false;

            // Check in shortage list, do some calculations if that item exists
            for mut entry in &mut shortages {
                if entry.pid == bom_list_entry.part_id {
                    // Calculate shortage based on known need plus new quantity
                    let mut short = entry.needed + bom_list_entry.quantity - inventory_quantity;

                    // Set short to 0 if > 0
                    if short < 0 {
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
                let part = find_part_by_id(&app.conn, &bom_list_entry.part_id);

                let part = match part {
                    Ok(x) => x,
                    Err(e) => return Err(e),
                };

                // Calculate the amount short
                let mut short = (build.quantity * bom_list_entry.quantity) - inventory_quantity;

                // To 0 if not short
                if short < 0 {
                    short = 0;
                }

                // Create shortage item
                let shortage = Shortage {
                    pid: bom_list_entry.part_id,
                    pn: part.pn,
                    mpn: part.mpn,
                    desc: part.descr,
                    have: inventory_quantity,
                    needed: build.quantity * bom_list_entry.quantity,
                    short,
                    unit_price: None,
                    notes: None,
                    quantity: None,
                };

                // Add to shortage list
                shortages.push(shortage);
            }
        }
    }

    // Remove items that are short = 0
    if !show_all_entries {
        let mut only_shortages: Vec<Shortage> = Vec::new();

        for shortage in shortages {
            if shortage.short != 0 {
                only_shortages.push(shortage);
            }
        }

        Ok(only_shortages)
    } else {
        //return the shortages
        Ok(shortages)
    }
}
