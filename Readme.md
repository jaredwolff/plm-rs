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
cargo run --bin mrp bom import -f /Users/jaredwolff/Documents/eagle/projects/pm25/pm25.sch
cargo run --bin mrp builds create
cargo run --bin mrp inventory show -s
cargo run --bin mrp inventory create -f test/inventory.csv
cargo run --bin mrp inventory show -s
cargo run --bin mrp inventory show
cargo run --bin mrp builds show -a
cargo run --bin mrp builds complete -b 1
```