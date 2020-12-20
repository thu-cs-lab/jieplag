#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub mod common;
pub mod env;
pub mod lang;
pub mod models;
pub mod schema;
pub mod session;
pub mod token;

pub type DbConnection = PgConnection;
type DbPool = Pool<ConnectionManager<DbConnection>>;
