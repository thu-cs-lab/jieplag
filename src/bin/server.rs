#[macro_use]
extern crate diesel_migrations;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::embed_migrations;
use dotenv::dotenv;
use env_logger;
use jieplag::{env::ENV, DbConnection};
use log::*;

embed_migrations!();

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("Bootstraping...");

    let url = ENV.database_url.clone();
    let manager = ConnectionManager::<DbConnection>::new(url);
    let pool = Pool::builder().build(manager)?;
    let conn = pool.get()?;
    embedded_migrations::run_with_output(&conn, &mut std::io::stdout())?;
    Ok(())
}
