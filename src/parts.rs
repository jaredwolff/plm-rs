extern crate diesel;
extern crate mrp;

use prettytable::Table;
use serde::Deserialize;

use self::models::*;
use self::mrp::*;
use diesel::prelude::*;
use std::io;

use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize)]
struct Record {
  pn: String,
  mpn: String,
  desc: String,
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

  let connection = establish_connection();

  // Get the input from stdin
  let pn = prompt.ask_text_entry("Part Number: ");
  let mpn = prompt.ask_text_entry("Manufacturer Part Number: ");
  let desc = prompt.ask_text_entry("Description: ");
  let ver = prompt.ask_text_entry("Version: ");
  let ver: i32 = ver.trim().parse().expect("Invalid version number!");

  // Create the part
  let part = NewUpdatePart {
    pn: &pn,
    mpn: &mpn,
    descr: &desc,
    ver: &ver,
    mqty: &1,
  };

  let found = find_part_by_pn(&connection, &pn);

  // If already found ask if it should be updated
  if found.is_ok() {
    let question = format!("{} already exists! Would you like to update it?", pn);
    let update = prompt.ask_yes_no_question(&question);

    // Update if they said yes.
    if update {
      update_part(&connection, &found.unwrap().id, &part).expect("Unable to update part!");

      // Check for success
      println!("{} updated!", pn);
    }
  } else {
    create_part(&connection, &part).expect("Unable to create part!");
  }
}

pub fn rename() {
  // For prompts
  let stdio = io::stdin();
  let input = stdio.lock();
  let output = io::stdout();

  let mut prompt = prompt::Prompt {
    reader: input,
    writer: output,
  };

  let connection = establish_connection();

  // Get the input from stdin
  let pn = prompt.ask_text_entry("Part Number: ");
  let newpn = prompt.ask_text_entry("New Part Number: ");

  rename_part(&connection, &pn, &newpn).expect("Unable to change pn");
}

pub fn create_by_csv(filename: &String) {
  // Establish connection!
  let conn = establish_connection();

  // For prompts
  let stdio = io::stdin();
  let input = stdio.lock();
  let output = io::stdout();

  let mut prompt = prompt::Prompt {
    reader: input,
    writer: output,
  };

  // Open the file
  let file = File::open(filename).unwrap();
  let file = BufReader::new(file);

  let mut records: Vec<Record> = Vec::new();

  let mut rdr = csv::Reader::from_reader(file);

  // TODO handle empty or malformed content a bit... better.
  // TODO: handle invalid data that's not UTF8
  for result in rdr.deserialize() {
    // Notice that we need to provide a type hint for automatic
    // deserialization.
    let record: Record = result.expect("Unable to deserialize.");
    println!("Processing: {:?}", record);
    records.push(record);
  }

  // Iterate through all the records.
  for record in records {
    // Create a new part from the CSV file
    let part = models::NewUpdatePart {
      pn: &record.pn,
      mpn: &record.mpn,
      descr: &record.desc,
      ver: &1,
      mqty: &1,
    };

    let found = find_part_by_pn(&conn, &part.pn);

    // If already found ask if it should be updated
    if found.is_ok() {
      let found = found.unwrap();

      // Compare the two make sure they're different
      if found.mpn != part.mpn || found.descr != part.descr || found.ver != *part.ver {
        let question = format!("{} already exists! Would you like to update it?", part.pn);

        // Create the table
        let mut table = Table::new();
        table.add_row(row![
          "Current:",
          found.pn,
          found.mpn,
          found.descr,
          found.ver
        ]);
        table.add_row(row!["Change to:", part.pn, part.mpn, part.descr, part.ver]);
        table.printstd();

        let update = prompt.ask_yes_no_question(&question);

        // Update if they said yes.
        if update {
          update_part(&conn, &found.id, &part).expect("Unable to update part!");

          // Check for success
          println!("{} updated!", part.pn);
        }
      }
    } else {
      println!("Creating: {:?}", part);
      create_part(&conn, &part).expect("Unable to create part!");
    }
  }
}

pub fn delete() {
  // For prompts
  let stdio = io::stdin();
  let input = stdio.lock();
  let output = io::stdout();

  let mut prompt = prompt::Prompt {
    reader: input,
    writer: output,
  };

  let part = prompt.ask_text_entry("Part Number: ");

  // First find the parts.
  let connection = establish_connection();
  let part = find_part_by_pn(&connection, &part).expect("Unable to find part!");

  // Then ask the user to confirm they want to delete
  let question = format!("Would you like to delete {}?", part.pn);
  let delete = prompt.ask_yes_no_question(&question);

  // THEN, delete if they said yes.
  if delete {
    // Delete the part
    let res = delete_part(&connection, &part.id);

    // Depending on the result show the feedback
    if res.is_err() {
      panic!("Error deleting part {}.", part.pn);
    } else {
      println!("Deleted {}", part.pn);
    }
  }
}

pub fn show() {
  use mrp::schema::*;

  // Create the table
  let mut table = Table::new();

  let connection = establish_connection();
  let results = parts::dsl::parts
    .load::<models::Part>(&connection)
    .expect("Error loading parts");

  println!("Displaying {} parts", results.len());
  table.add_row(row!["PN", "MPN", "Desc", "Mqty", "Ver"]);
  for part in results {
    table.add_row(row![part.pn, part.mpn, part.descr, part.mqty, part.ver]);
  }
  table.printstd();
}
