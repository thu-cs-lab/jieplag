pub mod env;
pub mod lang;
pub mod token;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbConnection = PgConnection;
type DbPool = Pool<ConnectionManager<DbConnection>>;
