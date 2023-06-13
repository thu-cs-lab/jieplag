use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbConnection = PgConnection;
pub(crate) type DbPool = Pool<ConnectionManager<DbConnection>>;
