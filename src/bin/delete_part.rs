extern crate mrp;
extern crate diesel;

use self::diesel::prelude::*;
use self::mrp::*;
use std::io::stdin;

fn main() {
  use mrp::schema::parts::dsl::*;

  println!("Part Number:");
  let mut part = String::new();
  stdin().read_line(&mut part).unwrap();
  let part = &part[..(part.len() - 1)]; // Drop the newline character
  let pattern = format!("%{}%", part);

  let connection = establish_connection();
  let num_deleted = diesel::delete(parts.filter(pn.like(pattern)))
      .execute(&connection)
      .expect("Error deleting parts");

  println!("Deleted {} parts", num_deleted);

}