use lazy_static::lazy_static;
use std::env::var;

pub struct Env {
    pub database_url: String,
    pub cookie_secret: String,
}

fn get_env() -> Env {
    Env {
        database_url: var("DATABASE_URL").expect("DATABASE_URL"),
        cookie_secret: var("COOKIE_SECRET").expect("COOKIE_SECRET"),
    }
}

lazy_static! {
    pub static ref ENV: Env = get_env();
}
