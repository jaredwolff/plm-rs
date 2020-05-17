## Migrations

`diesel migration run` to get to the latest
`diesel migration rever` to roll back

## Roadmap

* [x] Tests
* [x] Import a BOM from a .sch file
* [x] Adds/updates parts to DB from BOM import.
* [ ] Create BOM relationships
* [ ] Create a build based on a BOM
* [ ] Check shortages based on inventory
* [ ] Add inventory from CSV order

## Running a command

*Create a part manually*
`cargo run --bin create_part`

*Show all parts in db*
`cargo run --bin show_parts`

## Handy URLS

* <https://github.com/tafia/quick-xml>
* <https://www.techiediaries.com/sqlite-create-table-foreign-key-relationships/>