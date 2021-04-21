extern crate diesel;
extern crate quick_xml;
extern crate serde;

use crate::*;
use prettytable::Table;
use quick_xml::de::from_reader;

use self::diesel::prelude::*;

use std::fs::File;
use std::io::{self, BufReader};

#[derive(Eq, PartialEq)]
struct LineItem {
    name: String,
    pn: String,
    quantity: i32,
    nostuff: i32,
}

pub fn import(config: &config::Config, filename: &String) {
    use crate::schema::parts::dsl::*;

    // For prompts
    let stdio = io::stdin();
    let input = stdio.lock();
    let output = io::stdout();

    let mut prompt = prompt::Prompt {
        reader: input,
        writer: output,
    };

    // Establish connection!
    let conn = establish_connection(&config);

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

    // Serach for it and make sure that it matches
    let res = find_part_by_pn(&conn, &bom_pn);

    if res.is_ok() {
        // Unwrap from Option
        let bom = res.unwrap();

        let question = format!("BOM {} found! Would you like to update it?", bom_pn);
        let yes = prompt.ask_yes_no_question(&question);

        // If it already exists error/ask to update. Then runs the update routine instead
        if yes {
            // Ask if a new revision is requred
            let question = format!("BOM {} found! Would you like to up-rev the design?", bom_pn);
            let yes = prompt.ask_yes_no_question(&question);

            if yes {
                // Increment the version
                revision = bom.ver + 1;

                // Save the revision
                diesel::update(parts)
                    .set(ver.eq(revision))
                    .filter(id.eq(bom.id))
                    .execute(&conn)
                    .expect("Unable to update BOM revision!");
            } else {
                // Remove all previous BOM entries.
                delete_bom_list_by_id_and_ver(&conn, &bom.id, &bom.ver)
                    .expect("Unable to delete previous entries");
            }
        } else {
            // Exit then!
            std::process::exit(0);
        }
    } else {
        // Create new BOM part
        let part = models::NewUpdatePart {
            pn: &bom_pn,
            mpn: &bom_pn,
            descr: &bom_desc,
            ver: &revision,
            mqty: &1,
        };

        create_part(&conn, &part).expect("Unable to create BOM part!");
    }

    println!("Parts list:");
    let mut list: Vec<LineItem> = Vec::new();

    // Process the part list
    for part in &eagle.drawing.schematic.parts.part {
        // Check to make sure it's not GND, FIDUCIAL, MOUNTING, FRAME, +3V3
        let mut found = false;

        // TODO: a more comprehensive list or better way of filtering these..
        let check = [
            "GND", "FIDUCIAL", "MOUNTING", "FRAME", "+3V3", "TP", "VCC", "VBUS", "V5V0",
            "DOCFIELD", "VBAT", "VSYS", "PAD", "VDC",
        ];

        for entry in check.iter() {
            if part.deviceset.contains(entry) {
                found = true;
                break;
            }
        }

        // Continue loop if found
        if found {
            continue;
        }

        // Technology is optional. So need to do a match here.
        let technology = match &part.technology {
            Some(x) => x,
            None => "",
        };

        // Concatinate all the elements to form the actual part number
        let part_number = format!(
            "{}{}{}",
            part.deviceset,
            technology.to_string(),
            part.device,
        );

        // Check if it's no stuff. If so skip over adding it.
        let mut nostuff = 0;
        if part.variant.is_some() && part.variant.as_ref().unwrap().populate == "no" {
            nostuff = 1;
        }

        // Create temp line item
        let item = LineItem {
            name: part.name.clone(),
            pn: part_number,
            quantity: 1,
            nostuff: nostuff,
        };

        // TODO: Check if part has attribute (MQTY). This overrides MQTY from the library.

        // Check if list has
        let mut position = 0;
        let mut found = false;
        for (pos, part) in list.iter().enumerate() {
            // If the no stuff status and part number are the same
            // Then we've found the "same part"
            if part.pn == item.pn && part.nostuff == item.nostuff {
                position = pos;
                found = true;
                break;
            }
        }

        if !found {
            // If not found, add it
            list.push(item);
        } else if nostuff == 0 {
            // Increase the quantity
            list[position].name = format!("{} {}", list[position].name, item.name);
            list[position].quantity += 1;
        }
    }

    // Create the table for viewing
    let mut table = Table::new();
    table.add_row(row!["Part Number", "Refdes", "Quantity", "No Stuff"]);

    for item in &list {
        table.add_row(row![item.pn, item.name, item.quantity, item.nostuff]);
    }

    table.printstd();

    let mut table = Table::new();
    table.add_row(row![
        "Part Number",
        "MPN",
        "Description",
        "Multi Quantity",
        "Version"
    ]);

    // Get MPN, DigikeyPn from Library exerpts
    for mut item in list {
        // Temporary struct creation..
        #[derive(Debug)]
        struct Part {
            pn: String,
            mpn: String,
            descr: String,
            ver: i32,
            mqty: i32,
        }

        // Net part to add to the list
        let mut part = Part {
            pn: item.pn.clone(),
            mpn: "".to_string(),
            descr: "".to_string(),
            ver: 1,
            mqty: 1,
        };

        // If alias
        let mut is_alias = false;

        // Quantity local
        let mut mqty_temp: i32 = 1;

        // See if we're found
        let mut found;

        for library in &eagle.drawing.schematic.libraries.library {
            // Check if it's the library we care about.
            if library.name == config.library_name {
                'outer: for deviceset in &library.devicesets.deviceset {
                    for device in &deviceset.devices.device {
                        // Every new technology creates a new part.
                        for technology in &device.technologies.technology {
                            let library_part_number =
                                format!("{}{}{}", deviceset.name, technology.name, device.name);

                            // Check if found.
                            if library_part_number == item.pn {
                                found = true;
                            } else {
                                continue;
                            }

                            // If the part has attributes create
                            if technology.attribute.is_some() {
                                let attributes = technology.attribute.as_ref().unwrap();

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
                                        mqty_temp = attribute
                                            .value
                                            .trim()
                                            .parse()
                                            .expect("Unable to convert mqty");
                                    } else if attribute.name == "ALIAS" {
                                        // Set flag
                                        is_alias = true;

                                        // Get the part that this is aliasing..
                                        let alias = find_part_by_pn(&conn, &attribute.value);

                                        // Sort it out
                                        let alias = match alias {
                                            Ok(x) => x,
                                            Err(_) => {
                                                println!(
                                                    "Unable to find alias {}!",
                                                    attribute.value
                                                );
                                                std::process::exit(1);
                                            }
                                        };

                                        // Clone these bits so the live on
                                        part.pn = alias.pn.clone();
                                        part.mpn = alias.mpn.clone();
                                        part.descr = alias.descr.clone();
                                        part.ver = alias.ver;
                                        part.mqty = alias.mqty;
                                    }
                                }
                            }

                            // Break from everything
                            if found {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        // Set the quantity if it's *not* an alias
        if !is_alias {
            part.mqty = mqty_temp;
        } else {
            // If it is an alias, update mqty
            item.quantity = mqty_temp;
        }

        // Add to table view
        table.add_row(row![part.pn, part.mpn, part.descr, part.mqty, part.ver]);

        // Find part
        let existing = find_part_by_pn(&conn, &part.pn);

        if part.mpn == "" {
            println!("Manufacturer part number must be set for {}", part.pn);
            std::process::exit(1);
        }

        // Create update object
        let npart = models::NewUpdatePart {
            pn: &part.pn,
            mpn: &part.mpn,
            descr: &part.descr,
            ver: &part.ver,
            mqty: &part.mqty,
        };

        // Not found, create
        if existing.is_err() {
            println!("Creating: {:?}", npart);
            create_part(&conn, &npart).expect("Unable to create part!");
        } else {
            // Found, check for changes.
            let first = existing.unwrap();

            // Check for changes and ask if want to update.
            if npart.mpn != first.mpn
                || npart.descr != first.descr
                || *npart.ver != first.ver
                || *npart.mqty != first.mqty
            {
                let question = format!("{} found! Would you like to update it?", first.pn);

                // Create the table
                let mut table = Table::new();
                table.add_row(row![
                    "Current:",
                    first.pn,
                    first.mpn,
                    first.descr,
                    first.mqty,
                    first.ver
                ]);
                table.add_row(row![
                    "Change to:",
                    npart.pn,
                    npart.mpn,
                    npart.descr,
                    npart.mqty,
                    npart.ver
                ]);
                table.printstd();

                let yes = prompt.ask_yes_no_question(&question);

                if yes {
                    update_part(&conn, &first.id, &npart).expect("Error updating part!");
                }
            }
        }

        // Get the part ID
        let line_item = find_part_by_pn(&conn, &npart.pn);
        let bom_item = find_part_by_pn(&conn, &bom_pn);

        // Create BOM association between the part and the
        // BOM info like QTY, REFDES, NOSTUFF

        if line_item.is_ok() && bom_item.is_ok() {
            let line_item = line_item.unwrap();
            let bom_item = bom_item.unwrap();

            // Calcualte the qty using item qty and mqty
            let quantity = item.quantity * line_item.mqty;

            // Create the new relationship
            // ? Better way to do this?
            let relationship = models::NewPartsParts {
                quantity: &quantity,
                bom_ver: &bom_item.ver,
                refdes: &item.name,
                nostuff: &item.nostuff,
                bom_part_id: &bom_item.id,
                part_id: &line_item.id,
            };

            // Push them to the DB
            create_bom_line_item(&conn, &relationship).expect("Unable to add new BOM line item.");
        }
    }

    table.printstd();
}

pub fn show(config: &config::Config, part_number: &String, version: &Option<i32>) {
    use crate::schema::*;

    // Establish connection!
    let conn = establish_connection(&config);

    // Find the part
    let part = find_part_by_pn(&conn, &part_number);

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
    let results = parts_parts::dsl::parts_parts
        .filter(parts_parts::dsl::bom_part_id.eq(part.id))
        .filter(parts_parts::dsl::bom_ver.eq(ver))
        .load::<models::PartsPart>(&conn)
        .expect("Error loading parts");

    println!("Displaying {} parts", results.len());

    println!(
        "Part Number: {} BOM Id: {} Version: {}",
        part.pn, part.id, ver
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
        let details = find_part_by_id(&conn, &entry.part_id).expect("Unable to get details!");

        // Get inventory info
        let inventory = inventories::dsl::inventories
            .filter(inventories::dsl::part_id.eq(entry.part_id))
            .load::<models::Inventory>(&conn)
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

// pub fn delete(part_number: &String, version: &i32) {
//   // TODO: confirm exists
//   // TODO: confirm delete
//   // TODO delete query
// }
