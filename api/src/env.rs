use lazy_static::lazy_static;
use std::env::var;

pub struct Env {
    pub database_url: String,
    pub cookie_secret: String,
    pub public_url: String,
}

fn get_env() -> Env {
    Env {
        database_url: var("DATABASE_URL").expect("DATABASE_URL"),
        cookie_secret: var("COOKIE_SECRET").expect("COOKIE_SECRET"),
        public_url: var("PUBLIC_URL").expect("PUBLIC_URL"),
    }
}

lazy_static! {
    pub static ref ENV: Env = get_env();
}
