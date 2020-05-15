extern crate mrp;
extern crate diesel;

#[macro_use] extern crate prettytable;
use prettytable::Table;

use self::mrp::*;
use self::models::*;
use self::diesel::prelude::*;

fn main() {
    use mrp::schema::parts::dsl::*;

    // Create the table
    let mut table = Table::new();

    let connection = establish_connection();
    let results = parts
        .limit(5)
        .load::<Part>(&connection)
        .expect("Error loading parts");

    println!("Displaying {} parts", results.len());
    table.add_row(row!["PN","MPN","Desc","Ver"]);
    for part in results {
        table.add_row(row![part.pn,part.mpn,part.descr,part.ver]);
    }
    table.printstd();
}