extern crate diesel;
extern crate mrp;
extern crate quick_xml;
extern crate serde;

#[macro_use]
extern crate prettytable;
use prettytable::Table;

use quick_xml::de::from_reader;

use self::diesel::prelude::*;
use self::mrp::*;

use std::env;
use std::env::args;

use std::fs::File;
use std::io::{self, BufReader};

// Local includes
#[path = "../prompt.rs"]
mod prompt;
#[path = "../schematic.rs"]
mod schematic;

use prompt::Prompt;

#[derive(Eq, PartialEq)]
struct LineItem {
  name: String,
  pn: String,
  quantity: i32,
  nostuff: bool,
}

fn main() {
  use mrp::schema::parts::dsl::*;

  // For prompts
  let stdio = io::stdin();
  let input = stdio.lock();
  let output = io::stdout();

  let mut prompt = Prompt {
    reader: input,
    writer: output,
  };

  // Establish connection!
  let conn = establish_connection();

  // Takes a .sch file as an input
  let filename = args().nth(1).expect("Need a filename as an argument.");

  // Open the file
  let file = File::open(filename).unwrap();
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
      "GND", "FIDUCIAL", "MOUNTING", "FRAME", "+3V3", "TP", "VCC", "VBUS", "V5V0", "DOCFIELD",
      "VBAT",
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
    let mut nostuff = false;
    if part.variant.is_some() && part.variant.as_ref().unwrap().populate == "no" {
      nostuff = true;
    }

    // Create temp line item
    let item = LineItem {
      name: part.name.clone(),
      pn: part_number,
      quantity: 1,
      nostuff: nostuff,
    };

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
    } else if !nostuff {
      // Increase the quantity
      list[position].name = format!("{} {}", list[position].name, item.name);
      list[position].quantity += 1;
    }
  }

  // TODO: use list to create BOM setup

  // Create the table for viewing
  let mut table = Table::new();
  table.add_row(row!["Part Number", "Refdes", "Quantity", "No Stuff"]);

  for item in &list {
    table.add_row(row![item.pn, item.name, item.quantity, item.nostuff]);
  }

  table.printstd();

  let mut table = Table::new();
  table.add_row(row!["Part Number", "MPN", "Description", "Version"]);

  // Get MPN, DigikeyPn from Library exerpts
  for item in &list {
    // Net part to add to the list
    let mut part = models::NewUpdatePart {
      pn: &item.pn,
      mpn: "",
      descr: "",
      ver: &1,
    };

    let mut found;

    for library in &eagle.drawing.schematic.libraries.library {
      let default_lib =
        env::var("DEFAULT_LIBRARY_NAME").expect("DEFAULT_LIBRARY_NAME is not set in .env file!");

      // Check if it's the library we care about.
      if library.name == default_lib {
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
                  if attribute.name == "MPN" {
                    part.mpn = &attribute.value;
                  } else if attribute.name == "DIGIKEYPN" {
                    // part.digikeypn = &attribute.name;
                  } else if attribute.name == "DESC" {
                    part.descr = &attribute.value;
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

    // Add to table view
    table.add_row(row![part.pn, part.mpn, part.descr, part.ver]);

    // Find part
    let existing = find_part_by_pn(&conn, &part.pn);

    // Not found, create
    if existing.is_err() {
      create_part(&conn, &part).expect("Unable to create part!");
    } else {
      // Found, check for changes.
      let first = existing.unwrap();

      // Check for changes and ask if want to update.
      if part.mpn != first.mpn || part.descr != first.descr || *part.ver != first.ver {
        let question = format!("{} found! Would you like to update it?", first.pn);
        let yes = prompt.ask_yes_no_question(&question);

        if yes {
          update_part(&conn, &part).expect("Error updating part!");
        }
      }
    }

    // Get the part ID
    let line_item = find_part_by_pn(&conn, &part.pn);
    let bom_item = find_part_by_pn(&conn, &bom_pn);

    // Create BOM association between the part and the
    // BOM info like QTY, REFDES, NOSTUFF

    if line_item.is_ok() && bom_item.is_ok() {
      let line_item = line_item.unwrap();
      let bom_item = bom_item.unwrap();

      // Create the new relationship
      // ? Better way to do this?
      let relationship = models::NewPartsParts {
        quantity: &item.quantity,
        bom_ver: &bom_item.ver,
        refdes: &item.name,
        bom_part_id: &bom_item.id,
        part_id: &line_item.id,
      };

      // Push them to the DB
      create_bom_line_item(&conn, &relationship).expect("Unable to add new BOM line item.");
    }
  }

  table.printstd();
}
