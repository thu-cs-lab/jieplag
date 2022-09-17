use diesel::prelude::*;
use dotenv::dotenv;
use jieplag;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    user_name: String,

    #[structopt(short, long)]
    password: String,

    #[structopt(short, long)]
    force: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    dotenv().ok();
    let url = jieplag::env::ENV.database_url.clone();
    let mut conn = jieplag::DbConnection::establish(&url)?;

    let (salt, hash) = jieplag::session::hash(&args.password)?;
    let new_user = jieplag::models::NewUser {
        user_name: args.user_name,
        salt: Vec::from(salt),
        password: Vec::from(hash),
    };
    if args.force {
        diesel::insert_into(jieplag::schema::users::table)
            .values(&new_user)
            .on_conflict(jieplag::schema::users::user_name)
            .do_update()
            .set(&new_user)
            .execute(&mut conn)?;
    } else {
        diesel::insert_into(jieplag::schema::users::table)
            .values(&new_user)
            .execute(&mut conn)?;
    }
    Ok(())
}
