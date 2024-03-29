use actix_http::Uri;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    middleware,
    web::{self, JsonConfig},
    App, HttpServer,
};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use log::*;
use ring::digest;

use api::env::ENV;
use server::{
    db::DbConnection,
    render::{render_job, render_match, render_match_frame},
    session::login,
    submit::submit,
};

pub const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("Setup DB");
    let url = ENV.database_url.clone();
    let manager = ConnectionManager::<DbConnection>::new(url);
    let pool = Pool::builder().build(manager)?;
    let mut conn = pool.get()?;
    conn.run_pending_migrations(MIGRATIONS).unwrap();

    info!("Setup Server");
    let secret = ENV.cookie_secret.clone();
    let secret = digest::digest(&digest::SHA512, secret.as_bytes());
    let uri = ENV.public_url.parse::<Uri>()?;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(
                JsonConfig::default().limit(32 * 1024 * 1024 * 1024),
            )) // Enlarge body size limit
            .wrap(actix_cors::Cors::default().supports_credentials())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(secret.as_ref()),
                )
                .cookie_secure(true)
                .cookie_http_only(true)
                .build(),
            )
            .wrap(middleware::Logger::default())
            .service(
                web::scope(uri.path())
                    .service(web::scope("/api").service(login).service(submit))
                    .service(render_match)
                    .service(render_match_frame)
                    .service(render_job),
            )
    })
    .bind("0.0.0.0:8765")?
    .run()
    .await?;
    Ok(())
}
