# An Eagle Powered PLM + MRP for your command line!

This project utilizes the built-in capabilities of Eagle to store part data. Use this project to
capture your bill of materials. Use it to track inventories. Use it to create builds and
consume inventory accordingly! It's not meant to replace PLM systems like Aligni but rather
supplement product makers who may not be able to pay for it.

## Building

To build run `cargo build --release`. The release will be placed in `target/release`. As of this writing
the bin is called `mrp`.

## .env file

You do need an `.env` file. You should define your DB name/location and also the database you're using inside your schematic.

Example:

```
DATABASE_URL=./database.db
DEFAULT_LIBRARY_NAME=wolff-den
```

If you have parts distributed across many libraries, this solution will not work for you.

## Migrations

`diesel migration run` to get to the latest
`diesel migration revert` to roll back

## Roadmap

* [ ] Tests
* [x] Import a BOM from a .sch file
* [x] Adds/updates parts to DB from BOM import.
* [x] Create BOM relationships
* [x] Create a build based on a BOM
* [x] Check shortages based on inventory
* [x] Add inventory from CSV
* [ ] Update inventory from CSV
* [x] Complete a build and consume inventory
* [x] Unified binary
* [ ] Documentation

## Handy URLS

* <https://github.com/tafia/quick-xml>
* <https://www.techiediaries.com/sqlite-create-table-foreign-key-relationships/>

## Steps for testing

```
cargo run --bin mrp bom import -f /Users/jaredwolff/Documents/eagle/projects/pm25/pm25.sch
cargo run --bin mrp builds create
cargo run --bin mrp inventory show -s
cargo run --bin mrp inventory create -f test/inventory.csv
cargo run --bin mrp inventory show -s
cargo run --bin mrp inventory show
cargo run --bin mrp builds show -a
cargo run --bin mrp builds complete -b 1
```