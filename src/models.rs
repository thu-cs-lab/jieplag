use crate::schema::users;

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub user_name: String,
    pub salt: Vec<u8>,
    pub password: Vec<u8>,
}

#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub user_name: String,
    pub salt: Vec<u8>,
    pub password: Vec<u8>,
}
