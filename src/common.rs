use crate::token::Token;
use actix_web::{error::ErrorInternalServerError, Error};
use anyhow::anyhow;
use log::*;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Copy, Clone)]
pub struct LineMatch {
    pub left_from: u32,
    pub left_to: u32,
    pub right_from: u32,
    pub right_to: u32,
}

pub fn find_matches(left: &[Token], right: &[Token]) -> Vec<LineMatch> {
    let mut line_matches = vec![];
    let left_kind: Vec<u8> = left.iter().map(|t| t.kind).collect();
    let right_kind: Vec<u8> = right.iter().map(|t| t.kind).collect();
    let mut matches = rkr_gst::run(&left_kind, &right_kind, 40, 20);
    matches.sort_by_key(|m| m.pattern_index);

    for m in &matches {
        line_matches.push(LineMatch {
            left_from: left[m.pattern_index].line,
            left_to: left[m.pattern_index + m.length - 1].line,
            right_from: right[m.text_index].line,
            right_to: right[m.text_index + m.length - 1].line,
        });
    }

    // merge consecutive matches in line
    let mut i = 0;
    while i + 1 < line_matches.len() {
        if line_matches[i].left_to == line_matches[i + 1].left_from
            && line_matches[i].right_to == line_matches[i + 1].right_from
        {
            line_matches[i].left_to = line_matches[i + 1].left_to;
            line_matches[i].right_to = line_matches[i + 1].right_to;
            line_matches.drain(i + 1..i + 2);
        } else {
            i = i + 1;
        }
    }
    line_matches
}

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
