use actix_web::{error::ErrorInternalServerError, Error};
use anyhow::anyhow;
use log::*;
use uuid::Uuid;
use std::fmt::Display;

pub fn generate_uuid() -> String {
    let uuid = Uuid::new_v4();
    uuid.simple()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
}

#[track_caller]
pub fn err<T: Display>(err: T) -> Error {
    let error_token = generate_uuid();
    let location = std::panic::Location::caller();
    error!("Error {} at {}: {}", error_token, location, err);
    ErrorInternalServerError(anyhow!(format!(
        "Please contact admin with error token {}",
        error_token
    )))
}
