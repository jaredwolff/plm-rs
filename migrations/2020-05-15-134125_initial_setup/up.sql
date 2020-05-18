-- Your SQL goes here
CREATE TABLE parts (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  pn VARCHAR UNIQUE NOT NULL, -- part number
  mpn VARCHAR UNIQUE NOT NULL, -- manufacturer part number
  digikeypn VARCHAR UNIQUE, -- digikey part number
  descr VARCHAR NOT NULL, -- description
  ver INTEGER NOT NULL, -- version of part
  val VARCHAR, -- stores the part value (if any)
  created_at TIMESTAMP NOT NULL DEFAULT (datetime('now','localtime'))
);

-- Used to keep a ledger of all part inventory changes.
CREATE TABLE inventories (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  quantity INTEGER NOT NULL, -- how much there are
  unit_price REAL, -- the unit price
  notes TEXT, -- notes
  created_at TIMESTAMP  NOT NULL DEFAULT (datetime('now','localtime')),
  part_id INTEGER NOT NULL, -- the part that is associated with the inventory
  FOREIGN KEY(part_id) REFERENCES parts(id) --only one part associated with this inventory (many to one)
);

-- Used to track builds
CREATE TABLE builds (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT (datetime('now','localtime')),
  estimated_completion TIMESTAMP NOT NULL DEFAULT (datetime('now','localtime')),
  quantity INTEGER NOT NULL, -- how much there are
  cost REAL, -- cost per unit
  complete INTEGER NOT NULL, -- how much there are
  notes TEXT, -- text for build details
  part_ver INTEGER NOT NULL, -- version of the BOM we're using
  part_id INTEGER NOT NULL, -- the part/BOM we're building
  FOREIGN KEY(part_id) REFERENCES parts(id)
);

-- Used to track bom contents
CREATE TABLE parts_parts ( -- i.e. boms
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  quantity INTEGER NOT NULL, -- quantity that is used in this BOM
  bom_ver INTEGER NOT NULL, -- version of the bom that this is tied to
  refdes VARCHAR NOT NULL, -- tracking the refdes
  bom_part_id INTEGER NOT NULL, -- this is simply a part that has a BOM associated with it
  part_id INTEGER NOT NULL, -- this table has entries that are associated with individual parts.
  FOREIGN KEY(bom_part_id) REFERENCES parts(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY(part_id) REFERENCES parts(id) ON DELETE CASCADE ON UPDATE CASCADE
);