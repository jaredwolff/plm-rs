-- Your SQL goes here
CREATE TABLE parts (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  pn VARCHAR NOT NULL, -- part number
  mpn VARCHAR NOT NULL, -- manufacturer part number
  descr VARCHAR NOT NULL, -- description
  ver INTEGER NOT NULL, -- version of part
  created_at TIMESTAMP NOT NULL DEFAULT (datetime('now','localtime')),
  UNIQUE(pn,mpn) -- asserts uniqueness of this part
);

-- Used to keep a ledger of all part inventory changes.
CREATE TABLE inventories (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  quantity INTEGER NOT NULL, -- how much there are
  unit_price REAL NOT NULL, -- the unit price
  created_at TIMESTAMP  NOT NULL DEFAULT (datetime('now','localtime')),
  part_id INTEGER, -- the part that is associated with the inventory
  FOREIGN KEY(part_id) REFERENCES parts(id) --only one part associated with this inventory (many to one)
);

-- Used to track builds
CREATE TABLE builds (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT (datetime('now','localtime')),
  part_id INTEGER, -- the part/BOM we're building
  FOREIGN KEY(part_id) REFERENCES parts(id)
);

-- Used to track bom contents
CREATE TABLE parts_parts ( -- i.e. boms
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  quantity INTEGER NOT NULL, -- quantity that is used in this BOM
  bom_part_id INTEGER, -- this is simply a part that has a BOM associated with it
  part_id INTEGER, -- this table has entries that are associated with individual parts.
  FOREIGN KEY(bom_part_id) REFERENCES parts(id),
  FOREIGN KEY(part_id) REFERENCES parts(id)
);