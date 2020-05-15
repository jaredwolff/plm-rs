use super::schema::parts;

use chrono::NaiveDateTime;

#[derive(Identifiable,Queryable)]
pub struct Part {
    pub id: i32,
    pub pn: String,
    pub mpn: String,
    pub descr: String,
    pub ver: i32,
    pub created_at: NaiveDateTime
}

#[derive(Insertable)]
#[table_name="parts"]
pub struct NewPart<'a> {
    pub pn: &'a str,
    pub mpn: &'a str,
    pub descr: &'a str,
    pub ver: &'a i32,
}