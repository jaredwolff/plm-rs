use super::schema::parts;

use chrono::NaiveDateTime;

#[derive(Identifiable,Queryable)]
pub struct Part {
    pub id: i32,
    pub pn: String,
    pub mpn: String,
    pub digikeypn: Option<String>,
    pub descr: String,
    pub ver: i32,
    pub val: Option<String>,
    pub created_at: NaiveDateTime
}

#[derive(Eq,PartialEq,Debug,Insertable,AsChangeset)]
#[table_name="parts"]
pub struct NewUpdatePart<'a> {
    pub pn: &'a str,
    pub mpn: &'a str,
    pub descr: &'a str,
    pub ver: &'a i32,
}