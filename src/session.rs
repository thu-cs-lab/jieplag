use crate::{common::err, models::User, DbPool};
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Result};
use anyhow::anyhow;
use diesel::prelude::*;
use log::*;
use ring::{
    digest, pbkdf2,
    rand::{self, SecureRandom},
};
use serde::Deserialize;
use std::num::NonZeroU32;

const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
const NUM_ITER: u32 = 100_000;

pub fn hash(password: &str) -> anyhow::Result<([u8; CREDENTIAL_LEN], [u8; CREDENTIAL_LEN])> {
    let mut salt = [0u8; CREDENTIAL_LEN];
    let rng = rand::SystemRandom::new();
    rng.fill(&mut salt)
        .map_err(|_| anyhow!("Cannot generate random"))?;
    let mut hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        NonZeroU32::new(NUM_ITER).unwrap(),
        &salt,
        password.as_bytes(),
        &mut hash,
    );
    Ok((salt, hash))
}

pub fn verify(salt: &[u8], password: &str, hash: &[u8]) -> bool {
    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512,
        NonZeroU32::new(NUM_ITER).unwrap(),
        salt,
        password.as_bytes(),
        hash,
    )
    .is_ok()
}

#[cfg(test)]
mod tests {
    use super::{hash, verify};

    #[test]
    fn test_hash_password() {
        let password = "testtest";
        let (salt, hash) = hash(password).unwrap();
        assert!(verify(&salt, password, &hash));
        assert!(!verify(&salt, "testtest2", &hash));
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LoginRequest {
    pub user_name: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(
    session: Session,
    pool: web::Data<DbPool>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().map_err(err)?;
    use crate::schema::users::dsl;
    if let Ok(user) = dsl::users
        .filter(dsl::user_name.eq(&body.user_name))
        .first::<User>(&mut conn)
    {
        if verify(&user.salt, &body.password, &user.password) {
            session.insert("id", user.id)?;
            session.insert("user_name", &user.user_name)?;
            info!("User {} logged in", user.user_name);
            return Ok(HttpResponse::Ok().json(true));
        }
    }
    Ok(HttpResponse::Ok().json(false))
}
