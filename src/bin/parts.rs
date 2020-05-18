extern crate diesel;
extern crate mrp;

use prettytable::Table;

use self::models::*;
use self::mrp::*;
use diesel::prelude::*;
use std::io;

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
  };

  let res = create_part(&connection, &part);

  // Check for success
  let found = match res {
    Ok(_) => {
      println!("{} created!", pn);
      std::process::exit(0)
    }
    Err(_) => true,
  };

  // If already found ask if it should be updated
  if found {
    let question = format!("{} already exists! Would you like to update it?", pn);
    let update = prompt.ask_yes_no_question(&question);

    // Update if they said yes.
    if update {
      let res = update_part(&connection, &part);

      // Check for success
      match res {
        Ok(_) => println!("{} updated!", pn),
        Err(_) => {
          println!("unable to update {}", pn);
          std::process::exit(1);
        }
      };
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
    match res {
      Ok(_) => println!("{} deleted!", part.pn),
      Err(error) => panic!("Error deleting part {} => {:?}", part.pn, error),
    };
  }

  println!("Deleted {}", part.pn);
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
  table.add_row(row!["PN", "MPN", "Desc", "Ver"]);
  for part in results {
    table.add_row(row![part.pn, part.mpn, part.descr, part.ver]);
  }
  table.printstd();
}

// Prevent Visual Code from barfing
#[allow(dead_code)]
fn main() {}
