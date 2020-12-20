#[macro_use]
extern crate diesel_migrations;
use actix_session::CookieSession;
use actix_web::{middleware, web, App, HttpServer};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::embed_migrations;
use dotenv::dotenv;
use env_logger;
use jieplag::{env::ENV, session::login, DbConnection};
use log::*;
use ring::digest;

embed_migrations!();

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("Setup DB");
    let url = ENV.database_url.clone();
    let manager = ConnectionManager::<DbConnection>::new(url);
    let pool = Pool::builder().build(manager)?;
    let conn = pool.get()?;
    embedded_migrations::run_with_output(&conn, &mut std::io::stdout())?;

    info!("Setup Server");
    let secret = ENV.cookie_secret.clone();
    let secret = digest::digest(&digest::SHA512, secret.as_bytes());
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(actix_cors::Cors::default().supports_credentials())
            .wrap(
                CookieSession::private(secret.as_ref())
                    .secure(false)
                    .http_only(true)
                    .max_age(3600),
            )
            .wrap(middleware::Logger::default())
            .service(web::scope("/api").service(login))
    })
    .bind("127.0.0.1:8765")?
    .run()
    .await?;
    Ok(())
}
