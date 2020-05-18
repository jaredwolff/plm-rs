use super::schema::*;

use chrono::NaiveDateTime;

#[derive(Identifiable, Queryable)]
pub struct Part {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub pn: String,
    pub mpn: String,
    pub digikeypn: Option<String>,
    pub descr: String,
    pub ver: i32,
    pub val: Option<String>,
}

#[derive(Eq, PartialEq, Debug, Insertable, AsChangeset)]
#[table_name = "parts"]
pub struct NewUpdatePart<'a> {
    pub pn: &'a str,
    pub mpn: &'a str,
    pub descr: &'a str,
    pub ver: &'a i32,
}

#[derive(Identifiable, Queryable)]
pub struct PartsPart {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub quantity: i32,
    pub bom_ver: i32,
    pub refdes: String,
    pub nostuff: i32,
    pub bom_part_id: i32,
    pub part_id: i32,
}

#[derive(Eq, PartialEq, Debug, Insertable, AsChangeset)]
#[table_name = "parts_parts"]
pub struct NewPartsParts<'a> {
    pub quantity: &'a i32,
    pub bom_ver: &'a i32,
    pub refdes: &'a str,
    pub nostuff: &'a i32,
    pub bom_part_id: &'a i32,
    pub part_id: &'a i32,
}

#[derive(Identifiable, Queryable)]
#[table_name = "inventories"]
pub struct Inventory {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub quantity: i32,
    pub consumed: i32,
    pub unit_price: Option<f32>,
    pub notes: Option<String>,
    pub part_id: i32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "inventories"]
pub struct NewInventoryEntry<'a> {
    pub quantity: &'a i32,
    pub consumed: &'a i32,
    pub unit_price: Option<&'a f32>,
    pub part_id: &'a i32,
    pub notes: Option<&'a str>,
}

#[derive(Identifiable, Queryable)]
#[table_name = "builds"]
pub struct Build {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub estimated_completion: NaiveDateTime,
    pub quantity: i32,
    pub cost: Option<f32>,
    pub complete: i32,
    pub notes: Option<String>,
    pub part_ver: i32,
    pub part_id: i32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "builds"]
pub struct NewBuild<'a> {
    pub quantity: &'a i32,
    pub complete: &'a i32,
    pub notes: &'a str,
    pub part_ver: &'a i32,
    pub part_id: &'a i32,
}
