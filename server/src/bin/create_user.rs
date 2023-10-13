use api::env::ENV;
use clap::Parser;
use diesel::{Connection, RunQueryDsl};
use dotenv::dotenv;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    user_name: String,

    #[arg(short, long)]
    password: String,

    #[arg(short, long)]
    force: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dotenv().ok();
    let url = ENV.database_url.clone();
    let mut conn = server::db::DbConnection::establish(&url)?;

    let (salt, hash) = server::session::hash(&args.password)?;
    let new_user = server::models::NewUser {
        user_name: args.user_name,
        salt: Vec::from(salt),
        password: Vec::from(hash),
    };
    if args.force {
        diesel::insert_into(server::schema::users::table)
            .values(&new_user)
            .on_conflict(server::schema::users::user_name)
            .do_update()
            .set(&new_user)
            .execute(&mut conn)?;
    } else {
        diesel::insert_into(server::schema::users::table)
            .values(&new_user)
            .execute(&mut conn)?;
    }
    Ok(())
}
