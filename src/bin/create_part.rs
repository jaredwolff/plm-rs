extern crate mrp;
extern crate diesel;

use self::mrp::*;
use std::io::stdin;

fn main() {
  let connection = establish_connection();

  println!("Part Number:");
  let mut pn = String::new();
  stdin().read_line(&mut pn).unwrap();
  let pn = &pn[..(pn.len() - 1)]; // Drop the newline character

  println!("Manufacturer Part Number:");
  let mut mpn = String::new();
  stdin().read_line(&mut mpn).unwrap();
  let mpn = &mpn[..(mpn.len() - 1)]; // Drop the newline character

  println!("Description:");
  let mut desc = String::new();
  stdin().read_line(&mut desc).unwrap();
  let desc = &desc[..(desc.len() - 1)]; // Drop the newline character

  println!("Version:");
  let mut ver = String::new();
  stdin().read_line(&mut ver).unwrap();
  let ver:i32 = ver.trim().parse().expect("Invalid version number!");

  let _ = create_part(&connection, &pn, &mpn, &desc, &ver);
  println!("\nSaved part {}", pn);
}