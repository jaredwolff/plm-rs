extern crate diesel;
extern crate mrp;

use self::models::*;
use self::mrp::*;

use std::io;

fn main() {
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
    let entry = NewInventoryEntry {
      part_id: &part.id,
      unit_price: Some(&0.0),
      quantity: &adj,
      consumed: &0,
      notes: Some(&notes),
    };

    create_inventory(&conn, &entry).expect("Unable to create inventory item.");
  }
}
