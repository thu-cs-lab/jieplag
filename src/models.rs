use crate::schema::users;
#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub user_name: String,
    pub salt: Vec<u8>,
    pub password: Vec<u8>,
}
