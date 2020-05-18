## Migrations

`diesel migration run` to get to the latest
`diesel migration rever` to roll back

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
* [ ] Unified binary

## Running a command

*Create a part manually*
`cargo run --bin create_part`

*Show all parts in db*
`cargo run --bin show_parts`

## Handy URLS

* <https://github.com/tafia/quick-xml>
* <https://www.techiediaries.com/sqlite-create-table-foreign-key-relationships/>

## Steps for testing

```
cargo run --bin create_bom /Users/jaredwolff/Documents/eagle/projects/pm25/pm25.sch
cargo run --bin create_build PS-AQW 1 2
cargo run --bin show_inventory_shortage
cargo run --bin create_inventory_from_csv test/inventory.csv
cargo run --bin show_inventory_shortage
cargo run --bin show_inventory
cargo run --bin show_builds
cargo run --bin complete_build 1
```