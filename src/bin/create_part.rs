extern crate mrp;
extern crate diesel;

use self::mrp::*;
use std::io::{stdin,stdout,Write};

fn main() {
  let connection = establish_connection();

  print!("Part Number: ");
  stdout().flush().expect("Unable to flush output!");
  let mut pn = String::new();
  stdin().read_line(&mut pn).unwrap();
  let pn = &pn[..(pn.len() - 1)]; // Drop the newline character

  print!("Manufacturer Part Number: ");
  stdout().flush().expect("Unable to flush output!");
  let mut mpn = String::new();
  stdin().read_line(&mut mpn).unwrap();
  let mpn = &mpn[..(mpn.len() - 1)]; // Drop the newline character

  print!("Description: ");
  stdout().flush().expect("Unable to flush output!");
  let mut desc = String::new();
  stdin().read_line(&mut desc).unwrap();
  let desc = &desc[..(desc.len() - 1)]; // Drop the newline character

  print!("Version: ");
  stdout().flush().expect("Unable to flush output!");
  let mut ver = String::new();
  stdin().read_line(&mut ver).unwrap();
  let ver:i32 = ver.trim().parse().expect("Invalid version number!");

  // Create the part
  let res = create_part(&connection, &pn, &mpn, &desc, &ver);

  // Check for success
  let found = match res {
    Ok(_) => {println!("{} created!", pn); std::process::exit(0)},
    Err(_) => true,
  };

  // If already found ask if it should be updated
  if found  {
    print!("{} already exists! Would you like to update it? (y/n) ", pn);
    stdout().flush().expect("Unable to flush output!");

    // Parse input
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let ch = input.chars().next().unwrap();

    // Update if they said yes.
    if ch == 'y' {

      let res = update_part(&connection, &pn, &mpn, &desc, &ver);

      // Check for success
      match res {
        Ok(_) => println!("{} updated!", pn),
        Err(_) => {println!("unable to update {}",pn);std::process::exit(1);},
      };

    }

  }

}