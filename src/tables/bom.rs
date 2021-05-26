extern crate diesel;
extern crate quick_xml;
extern crate serde;

use crate::schematic::VariantDef;
use crate::*;
use prettytable::Table;
use quick_xml::de::from_reader;

use self::diesel::prelude::*;

use chrono::Utc;
use serde::Serialize;
use std::io::{BufReader, BufWriter};
use std::{
    fs::File,
    io::{StdinLock, Stdout},
};

#[derive(Eq, PartialEq, Debug)]
struct LineItem {
    name: String,
    pn: String,
    quantity: i32,
    nostuff: i32,
}

#[derive(Debug, Default, Clone)]
struct SimplePart {
    pn: String,
    mpn: String,
    descr: String,
    ver: i32,
    mqty: i32,
    nostuff: i32,
}

#[derive(Serialize)]
struct BomEntry {
    pn: String,
    quantity: i32,
    refdes: String,
    mpn: String,
    descr: String,
    ver: i32,
    inventory_qty: i32,
    no_stuff: i32,
}

/// Helper function that prints a list of SimpleParts
fn print_simple_part_list(list: &Vec<SimplePart>) {
    let mut table = Table::new();
    table.add_row(row![
        "PART NUMBER",
        "MPN",
        "DESCRIPTION",
        "MULTI QUANTITY",
        "VERSION",
        "NO STUFF"
    ]);

    for part in list {
        table.add_row(row![
            part.pn,
            part.mpn,
            part.descr,
            part.mqty,
            part.ver,
            part.nostuff
        ]);
    }

    table.printstd();
}

// /// Helper function that prints a list of LineItems
// fn print_line_items(list: &Vec<LineItem>) {
//     // Create the table for viewing
//     let mut table = Table::new();
//     table.add_row(row!["Part Number", "Refdes", "Quantity", "No Stuff"]);

//     for item in list {
//         table.add_row(row![item.pn, item.name, item.quantity, item.nostuff]);
//     }

//     table.printstd();
// }

/// Updates a SimplePart based on attribute data from the library.
fn get_simplepart_from_library(
    item: &LineItem,
    eagle: &schematic::Eagle,
    library_name: &str,
) -> SimplePart {
    // Flags
    // let mut found = false;
    // let mut is_alias = false;

    // Quantity local
    // let mut mqty_temp: i32 = 1;
    let mut part: SimplePart = Default::default();

    // Set PN
    part.pn = item.pn.clone();
    part.nostuff = item.nostuff;
    part.mqty = item.quantity;

    for library in &eagle.drawing.schematic.libraries.library {
        // Check if it's the library we care about.
        if library.name == library_name {
            'outer: for deviceset in &library.devicesets.deviceset {
                for device in &deviceset.devices.device {
                    // Every new technology creates a new part.
                    for technology in &device.technologies.technology {
                        let library_part_number =
                            format!("{}{}{}", deviceset.name, technology.name, device.name);

                        // Check if found.
                        if library_part_number == item.pn {
                            match &technology.attribute {
                                Some(attributes) => {
                                    // Get the attributes we care about.
                                    for attribute in attributes {
                                        // Blank value check
                                        if attribute.value == "" {
                                            continue;
                                        }

                                        if attribute.name == "MPN" {
                                            part.mpn = attribute.value.clone();
                                        } else if attribute.name == "DIGIKEYPN" {
                                            // part.digikeypn = &attribute.name;
                                        } else if attribute.name == "DESC" {
                                            part.descr = attribute.value.clone();
                                        } else if attribute.name == "MQTY" {
                                            // Convert to int
                                            part.mqty = attribute
                                                .value
                                                .trim()
                                                .parse()
                                                .expect("Unable to convert mqty");
                                        } else if attribute.name == "ALIAS" {
                                            println!("Alias!");
                                            // // Set flag
                                            // is_alias = true;

                                            // // Get the part that this is aliasing..
                                            // let alias = find_part_by_pn(&app.conn, &attribute.value);

                                            // // Sort it out
                                            // let alias = match alias {
                                            //     Ok(x) => x,
                                            //     Err(_) => {
                                            //         println!("Unable to find alias {}!", attribute.value);
                                            //         std::process::exit(1);
                                            //     }
                                            // };

                                            // // Clone these bits so the live on
                                            // part.pn = alias.pn.clone();
                                            // part.mpn = alias.mpn.clone();
                                            // part.descr = alias.descr.clone();
                                            // part.ver = alias.ver;
                                            // part.mqty = alias.mqty;
                                        }
                                    }
                                }
                                None => (),
                            };

                            // Break the outer loop since we found it
                            break 'outer;
                        }
                    }
                }
            }
        };
    }

    // Return it
    part

    // Set the quantity if it's *not* an alias
    // if !is_alias {
    //     part.mqty = mqty_temp;
    // } else {
    //     // If it is an alias, update mqty
    //     item.quantity = mqty_temp;
    // }
}

/// Using a list of parts, this function determines the line items for a BOM
fn get_line_items_from_parts(
    parts: &Vec<schematic::Part>,
    variant: &VariantDef,
    ignore_list: &Vec<String>,
) -> Vec<LineItem> {
    let mut list: Vec<LineItem> = Vec::new();

    // Process the part list
    'outer: for part in parts {
        // Check to make sure it's not GND, FIDUCIAL, MOUNTING, FRAME, +3V3, etc
        for entry in ignore_list.iter() {
            if part.deviceset.contains(entry) {
                continue 'outer;
            }
        }

        // Technology is optional. So need to do a match here.
        let mut technology = part.technology.clone().unwrap_or_default();

        // Check if it's no stuff. If so skip over adding it.
        let mut nostuff = 0;
        for var in &part.variants {
            if var.name == variant.name {
                // Only update the technology if it exists!
                technology = match &var.technology {
                    Some(t) => t.to_string(),
                    None => technology,
                };

                // Set no stuff
                if var.populate == Some("no".to_string()) {
                    nostuff = 1;
                }
            }
        }

        // Concatinate all the elements to form the actual part number
        let part_number = format!("{}{}{}", part.deviceset, technology, part.device,);

        // Create temp line item
        let item = LineItem {
            name: part.name.clone(),
            pn: part_number,
            quantity: 1,
            nostuff: nostuff,
        };

        // TODO: Check if part has attribute (MQTY). This overrides MQTY from the library.

        // Check if list has
        let mut found = false;
        for entry in list
            .iter_mut()
            .filter(|part| part.pn == item.pn && part.nostuff == item.nostuff)
        {
            found = true;

            // Increase the quantity
            entry.name = format!("{} {}", entry.name, item.name);
            entry.quantity += 1;

            // Should only process once. All parts with same pn should be gathered together
            break;
        }

        // Only add to the list if it was found and not nostuff
        if !found {
            // If not found, add it
            list.push(item);
        }
    }

    // Return the list
    list
}

/// Compares a 'new' part with an existing Part. Prompts for user input.
/// Responds witha boolean value of whether or not to update.
fn prompt_to_update_part(
    prompt: &mut prompt::Prompt<StdinLock, Stdout>,
    new: &models::NewUpdatePart,
    existing: &models::Part,
) -> bool {
    // Check for changes and ask if want to update.
    if new.mpn != existing.mpn || new.descr != existing.descr || *new.ver != existing.ver {
        let question = format!("{} found! Would you like to update it?", existing.pn);

        // Create the table
        let mut table = Table::new();
        table.add_row(row!["", "pn", "mpn", "decr", "mqty", "ver"]);
        table.add_row(row![
            "Current:",
            existing.pn,
            existing.mpn,
            existing.descr,
            existing.mqty,
            existing.ver
        ]);
        table.add_row(row![
            "Change to:",
            new.pn,
            new.mpn,
            new.descr,
            new.mqty,
            new.ver
        ]);
        table.printstd();

        // Return response to question
        return prompt.ask_yes_no_question(&question);
    }

    // Otherwise return false
    false
}

/// Function used to import parts from file
pub fn import(app: &mut crate::Application, filename: &String) {
    use crate::schema::parts::dsl::*;

    // Open the file
    let file = File::open(filename);

    // Make sure it's valid
    let file = match file {
        Ok(x) => x,
        Err(_) => {
            println!("Unable to open {}", filename);
            std::process::exit(1);
        }
    };

    let file = BufReader::new(file);
    let eagle: schematic::Eagle = from_reader(file).expect("error parsing xml");

    // println!("{:?}",eagle);
    // let mut list: Vec<String> = [].to_vec();

    let mut found = false;
    let mut bom_pn = "".to_string();
    let mut bom_desc = "".to_string();
    let mut revision = 1;

    // Parses it to make sure it has a global variable defining the part # for the assembly
    for attribute in &eagle.drawing.schematic.attributes.attribute {
        // Get the part description
        if attribute.name == "DESC" {
            bom_desc = attribute.value.clone();
            println!("Desc: {}", bom_desc);
        }

        // Get the part name
        if attribute.name == "PN" {
            found = true;
            bom_pn = attribute.value.clone();
            println!("Part name: {}", bom_pn);
        }
    }

    // Warning about blank description
    if bom_desc == "" {
        println!("Warning: Blank BOM description");
    }

    // Error if PN is not found
    if !found {
        println!("Please add PN attribute to schematic!");
        std::process::exit(1);
    }

    // Get the variant list
    let mut variant: Option<VariantDef> = None;
    for v in &eagle.drawing.schematic.variantdefs.variantdef {
        // Get the current def
        if v.current == Some("yes".to_string()) {
            println!("Variant: {}", v.name);
            variant = Some(v.clone());
            break;
        }
    }

    // Error if not active variant
    // TODO: or prompt for variant..
    let variant = match variant {
        Some(v) => v,
        None => {
            println!("Error: no active variant!");
            std::process::exit(1);
        }
    };

    // Serach for it and make sure that it matches
    let res = find_part_by_pn(&app.conn, &bom_pn);

    // TODO: Check if there is inventory by part number for this part

    // TODO: if no inventory, create 0 inventory

    // Skip a line
    println!("");

    // Different actions depending if it exists
    match res {
        Ok(bom) => {
            let question = format!("BOM {} found! Would you like to update it?", bom_pn);
            let yes = app.prompt.ask_yes_no_question(&question);

            // If it already exists error/ask to update. Then runs the update routine instead
            if yes {
                // Ask if a new revision is requred
                let question =
                    format!("BOM {} found! Would you like to up-rev the design?", bom_pn);
                let yes = app.prompt.ask_yes_no_question(&question);

                if yes {
                    // Increment the version
                    revision = bom.ver + 1;

                    // Save the revision
                    diesel::update(parts)
                        .set(ver.eq(revision))
                        .filter(id.eq(bom.id))
                        .execute(&app.conn)
                        .expect("Unable to update BOM revision!");
                } else {
                    // Remove all previous BOM entries.
                    delete_bom_list_by_id_and_ver(&app.conn, &bom.id, &bom.ver)
                        .expect("Unable to delete previous entries");
                }
            } else {
                // Exit then!
                std::process::exit(0);
            }
        }
        Err(_) => {
            // Create new BOM part
            let part = models::NewUpdatePart {
                pn: &bom_pn,
                mpn: &bom_pn,
                descr: &bom_desc,
                ver: &revision,
                mqty: &1,
            };

            create_part(&app.conn, &part).expect("Unable to create BOM part!");
        }
    }

    println!("\nPARTS LIST:");
    let list = get_line_items_from_parts(
        &eagle.drawing.schematic.parts.part,
        &variant,
        &app.config.part_number_ignore_list,
    );

    // Vector of SimpleParts
    let mut simple_part_list: Vec<SimplePart> = Vec::new();

    // Get MPN, DigikeyPn from Library exerpts
    for item in list {
        // Set part attributes from library
        let part = get_simplepart_from_library(&item, &eagle, &app.config.library_name);

        if part.mpn == "" {
            println!("Manufacturer part number must be set for {}", part.pn);
            std::process::exit(1);
        }

        // Add to list
        simple_part_list.push(part.clone());

        // Find part
        let existing = find_part_by_pn(&app.conn, &part.pn);

        // Create update object
        let npart = models::NewUpdatePart {
            pn: &part.pn,
            mpn: &part.mpn,
            descr: &part.descr,
            ver: &part.ver,
            mqty: &part.mqty,
        };

        // Not found, create
        match existing {
            Ok(e) => {
                // Check if can be updated
                if prompt_to_update_part(&mut app.prompt, &npart, &e) {
                    update_part(&app.conn, &e.id, &npart).expect("Error updating part!");
                }
            }
            Err(_) => {
                println!("Creating: {:?}", npart);
                create_part(&app.conn, &npart).expect("Unable to create part!");
            }
        }

        // Get the part ID
        let line_item = find_part_by_pn(&app.conn, &npart.pn);
        let bom_item = find_part_by_pn(&app.conn, &bom_pn);

        // Create BOM association between the part and the
        // BOM info like QTY, REFDES, NOSTUFF

        match (line_item, bom_item) {
            (Ok(li), Ok(bi)) => {
                // Create the new relationship
                let relationship = models::NewPartsParts {
                    quantity: &item.quantity,
                    bom_ver: &bi.ver,
                    refdes: &item.name,
                    nostuff: &item.nostuff,
                    bom_part_id: &bi.id,
                    part_id: &li.id,
                };

                // Push them to the DB
                create_bom_line_item(&app.conn, &relationship)
                    .expect("Unable to add new BOM line item.");
            }
            _ => (),
        };
    }

    // Print out in simple part list format
    print_simple_part_list(&simple_part_list);
}

/// Function used to show parts in BOM
pub fn show(app: &mut crate::Application, part_number: &String, version: &Option<i32>) {
    use crate::schema::*;

    // Find the part
    let part = find_part_by_pn(&app.conn, &part_number);

    if part.is_err() {
        println!("{} was not found!", part_number);
        std::process::exit(1);
    }

    // Transform the response into a Part
    let part = part.unwrap();

    // Create the table
    let mut table = Table::new();

    // Then either use the provided version or the latest
    let ver = match version {
        Some(x) => x,
        None => &part.ver,
    };

    // Get all the parts related to this BOM
    let mut results = parts_parts::dsl::parts_parts
        .filter(parts_parts::dsl::bom_part_id.eq(part.id))
        .filter(parts_parts::dsl::bom_ver.eq(ver))
        .load::<models::PartsPart>(&app.conn)
        .expect("Error loading parts");

    // Sort the results by refdes
    results.sort_by(|a, b| a.refdes.cmp(&b.refdes));

    println!("Displaying {} parts", results.len());

    println!(
        "Part Number: {} BOM Id: {} Version: {}",
        part.pn, part.id, ver
    );

    table.add_row(row![
        "QUANTITY",
        "REFDES",
        "PN",
        "MPN",
        "DESC",
        "VER",
        "INVENTORY QTY",
        "NO STUFF"
    ]);
    for entry in results {
        let details = find_part_by_id(&app.conn, &entry.part_id).expect("Unable to get details!");

        // Get inventory info
        let inventory = inventories::dsl::inventories
            .filter(inventories::dsl::part_id.eq(entry.part_id))
            .load::<models::Inventory>(&app.conn)
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
            inventory_qty,
            entry.nostuff,
        ]);
    }
    table.printstd();
}

/// Function used to export BOM to CSV
pub fn export(app: &mut crate::Application, part_number: &str, version: &Option<i32>) {
    use crate::schema::*;

    // Find the part
    let part = find_part_by_pn(&app.conn, &part_number);

    if part.is_err() {
        println!("{} was not found!", part_number);
        std::process::exit(1);
    }

    // Transform the response into a Part
    let part = part.unwrap();

    // Then either use the provided version or the latest
    let ver = match version {
        Some(x) => x,
        None => &part.ver,
    };

    // Get all the parts related to this BOM
    let mut results = parts_parts::dsl::parts_parts
        .filter(parts_parts::dsl::bom_part_id.eq(part.id))
        .filter(parts_parts::dsl::bom_ver.eq(ver))
        .load::<models::PartsPart>(&app.conn)
        .expect("Error loading parts");

    // Sort the results by refdes
    results.sort_by(|a, b| {
        let first = a.refdes.chars().nth(0).unwrap();
        let second = b.refdes.chars().nth(0).unwrap();
        first.cmp(&second)
    });

    // Create filename
    let filename = format!("{}-v{}-{}.csv", part_number, ver, Utc::now().to_rfc3339());

    // File operations
    let file = File::create(&filename).unwrap();
    let file = BufWriter::new(file);

    // Create CSV writer
    let mut wtr = csv::Writer::from_writer(file);

    for entry in results {
        let details = find_part_by_id(&app.conn, &entry.part_id).expect("Unable to get details!");

        // Get inventory info
        let inventory = inventories::dsl::inventories
            .filter(inventories::dsl::part_id.eq(entry.part_id))
            .load::<models::Inventory>(&app.conn)
            .expect("Error loading parts");

        let mut inventory_qty = 0;

        // Tally inventory
        for item in inventory {
            // Increment quantity
            inventory_qty += item.quantity;
        }

        // Then pop it into a serializeable struct
        let line = BomEntry {
            quantity: entry.quantity,
            refdes: entry.refdes,
            pn: details.pn,
            mpn: details.mpn,
            descr: details.descr,
            ver: details.ver,
            inventory_qty,
            no_stuff: entry.nostuff,
        };

        wtr.serialize(line).expect("Unable to serialize.");
        wtr.flush().expect("Unable to flush");
    }

    println!("Inventory list exported to {}", filename);
}

// pub fn delete(part_number: &String, version: &i32) {
//   // TODO: confirm exists
//   // TODO: confirm delete
//   // TODO delete query
// }
