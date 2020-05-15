extern crate mrp;
extern crate diesel;

use self::diesel::prelude::*;
use self::models::*;
use self::mrp::*;
use std::io::{stdin,stdout,Write};

fn main() {
  use mrp::schema::parts::dsl::*;

  let mut num_deleted = 0;

  print!("Part Number: ");
  stdout().flush().expect("Unable to flush output!");
  let mut part = String::new();
  stdin().read_line(&mut part).unwrap();
  let part = &part[..(part.len() - 1)]; // Drop the newline character
  let pattern = format!("%{}%", part);

  // First find the parts.
  let connection = establish_connection();
  let results = parts
  .filter(pn.like(pattern))
  .load::<Part>(&connection)
  .expect("Error loading parts");

  for part in results {
    // Then ask the user to confirm they want to delete
    print!("Would you like to delete {}? (y/n) ", part.pn);
    stdout().flush().expect("Unable to flush output!");

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let ch = input.chars().next().unwrap();

    // THEN, delete if they said yes.
    if ch == 'y' {

      // Delete the part
      let res = diesel::delete(parts.filter(id.eq(part.id)))
      .execute(&connection);

      // Depending on the result show the feedback
      match res {
        Ok(_) => println!("{} deleted!", part.pn),
        Err(error) => panic!("Error deleting part {} => {:?}", part.pn, error),
      };

      // Increment num_deleted
      num_deleted+=1;
    }

  }

  println!("Deleted {} parts", num_deleted);

}