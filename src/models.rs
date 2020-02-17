use super::schema::whims;

#[derive(Insertable, Queryable)]
#[table_name="whims"]
pub struct Whim {
    pub title: String,
    pub body: String,
}
