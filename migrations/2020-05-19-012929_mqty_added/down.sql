-- This file should undo anything in `up.sql`
CREATE TABLE new_parts (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT (datetime('now','localtime')),
  updated_at TIMESTAMP NOT NULL DEFAULT (datetime('now','localtime')),
  pn VARCHAR UNIQUE NOT NULL, -- part number
  mpn VARCHAR UNIQUE NOT NULL, -- manufacturer part number
  digikeypn VARCHAR UNIQUE, -- digikey part number
  descr VARCHAR NOT NULL, -- description
  ver INTEGER NOT NULL, -- version of part
  val VARCHAR -- stores the part value (if any)
);

INSERT INTO new_parts SELECT id, created_at, updated_at, pn, mpn, digikeypn, descr, ver, val FROM parts;
DROP TABLE IF EXISTS parts;
ALTER TABLE new_parts RENAME TO parts;