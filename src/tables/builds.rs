extern crate diesel;

use chrono::Utc;
use prettytable::Table;

use self::diesel::prelude::*;
use crate::{models::*, *};

// Borrowing shortage generation from inventory
use super::inventory;

use serde::{Deserialize, Serialize};

use std::{
    fs::File,
    io::{self, BufWriter},
};

pub fn create(app: &mut crate::Application) {
    // Get the input from stdin
    let part_number = app.prompt.ask_text_entry("Part Number: ");
    let version = app.prompt.ask_text_entry("Version: ");
    let version: i32 = version.trim().parse().expect("Invalid version number!");
    let quantity = app.prompt.ask_text_entry("Quantity: ");
    let quantity: i32 = quantity.trim().parse().expect("Invalid quantity!");

    let part = find_part_by_pn(&app.conn, &part_number);

    if part.is_err() {
        println!("{} version {} was not found!", part_number, version);
        std::process::exit(1);
    }

    // Transform the response into a Part
    let part = part.unwrap();

    if part.ver != version {
        println!(
            "{} version {} was not found! Latest is: {}",
            part_number, version, part.ver
        );
        std::process::exit(1);
    }

    let build = NewUpdateBuild {
        quantity: &quantity,
        complete: &0,
        notes: Some(""),
        part_ver: &version,
        part_id: &part.id,
    };

    create_build(&app.conn, &build).expect("Unable to create build!");

    println!(
        "Created build of {} ver: {} with qty: {}",
        part.pn, part.ver, quantity
    );
}

pub fn show(app: &mut crate::Application, show_all: bool) {
    use crate::schema::*;

    // Create the table
    let mut table = Table::new();

    let results: Vec<Build>;

    if show_all {
        results = builds::dsl::builds
            .load::<models::Build>(&app.conn)
            .expect("Error loading builds");
    } else {
        results = builds::dsl::builds
            .filter(builds::dsl::complete.eq(0))
            .load::<models::Build>(&app.conn)
            .expect("Error loading builds");
    }

    println!("Displaying {} builds", results.len());
    table.add_row(row![
        "Build ID", "PN", "Ver", "Notes", "Complete", "Quantity"
    ]);
    for build in results {
        // Get the part info..
        let part = find_part_by_id(&app.conn, &build.part_id).expect("Unable to get build part.");
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

pub fn delete(app: &mut crate::Application, build_id: i32) {
    delete_build(&app.conn, &build_id).expect("Unable to delete build.");

    println!("Deleted build id: {} successfully!", build_id);
}

pub fn complete(app: &mut crate::Application, build_id: i32) {
    use crate::schema::*;

    // Get the build
    let build = find_build_by_id(&app.conn, &build_id).expect("Unable to find build!");

    // Get partslist
    let bom_list = parts_parts::dsl::parts_parts
        .filter(parts_parts::dsl::bom_part_id.eq(build.part_id))
        .filter(parts_parts::dsl::bom_ver.eq(build.part_ver))
        .load::<PartsPart>(&app.conn)
        .expect("Error loading parts");

    // Get the shortages. Shorts only.
    let shortages = inventory::get_shortages(app, false).expect("Unable to get shortages.");

    // Still track if we're short.
    let mut still_short = false;

    // Make sure that all parts are not short.
    for entry in shortages {
        if entry.short != 0 {
            println!("Still short {} for part: {}", entry.needed, entry.pn,);
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
            let inventory_entries = find_inventories_by_part_id(&app.conn, &bom_list_entry.part_id)
                .expect("Unable to query for inventory");

            // Calculate the quantity
            for entry in inventory_entries {
                let new_qty;
                let used;

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
                update_inventory_by_id(&app.conn, &entry.id, &update)
                    .expect("Unable to create inventory.");

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
        update_build_by_id(&app.conn, &build.id, &update_build).expect("Unable to update build!");

        // Push this inventory item
        create_inventory(&app.conn, &new_inventory).expect("Unable to create inventory.");
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildExport {
    pn: String,
    mpn: String,
    desc: String,
    quantity_in_stock: i32,
    quantity_needed: i32,
    checked: Option<bool>,
}

/// Function used to export BOM to CSV
pub fn export(app: &mut crate::Application, build_id: i32) {
    use crate::schema::*;

    // Get the build
    let build = find_build_by_id(&app.conn, &build_id).expect("Unable to find build!");

    // Get build part number
    let build_pn = find_part_by_id(&app.conn, &build.part_id).expect("Unable to get part by id");

    // Get partslist
    let bom_list = parts_parts::dsl::parts_parts
        .filter(parts_parts::dsl::bom_part_id.eq(build.part_id))
        .filter(parts_parts::dsl::bom_ver.eq(build.part_ver))
        .load::<PartsPart>(&app.conn)
        .expect("Error loading parts");

    // Create filename
    let filename = format!(
        "{}-v{}-BUILD={}-{}.csv",
        build_pn.pn,
        build.part_ver,
        build.id,
        Utc::now().to_rfc3339()
    );

    // File operations
    let file = File::create(&filename).unwrap();
    let file = BufWriter::new(file);

    // Create CSV writer
    let mut wtr = csv::Writer::from_writer(file);

    // Iterate though every bom list entry
    // Do the math to modify the inventory
    for bom_list_entry in &bom_list {
        // Skip if nostuff is set
        if bom_list_entry.nostuff == 1 {
            println!("{} is no stuff.", bom_list_entry.refdes);
            continue;
        }

        // Get part
        let part =
            find_part_by_id(&app.conn, &bom_list_entry.part_id).expect("Unable to get part by id");

        // Inventory entries
        let inventory_entries = find_inventories_by_part_id(&app.conn, &bom_list_entry.part_id)
            .expect("Unable to query for inventory");

        let quantity_available: i32 = inventory_entries.iter().map(|x| x.quantity).sum();

        let line = BuildExport {
            pn: part.pn,
            mpn: part.mpn,
            desc: part.descr,
            quantity_in_stock: quantity_available,
            quantity_needed: bom_list_entry.quantity * build.quantity,
            checked: None,
        };

        wtr.serialize(line).expect("Unable to serialize.");
        wtr.flush().expect("Unable to flush");
    }

    println!("Build exported to {}", filename);
}
