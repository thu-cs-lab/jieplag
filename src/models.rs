use crate::schema::{blocks, jobs, matches, submissions, users};

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub user_name: String,
    pub salt: Vec<u8>,
    pub password: Vec<u8>,
}

#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub user_name: String,
    pub salt: Vec<u8>,
    pub password: Vec<u8>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = jobs)]
pub struct NewJob {
    pub creator_user_id: i32,
    pub slug: String,
}

#[derive(Debug, Queryable)]
pub struct Job {
    pub id: i32,
    pub creator_user_id: i32,
    pub slug: String,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = matches)]
pub struct NewMatch {
    pub job_id: i32,
    pub left_submission_id: i32,
    pub left_match_rate: i32,
    pub right_submission_id: i32,
    pub right_match_rate: i32,
    pub lines_matched: i32,
}

#[derive(Debug, Queryable)]
pub struct Match {
    pub id: i32,
    pub job_id: i32,
    pub left_submission_id: i32,
    pub left_match_rate: i32,
    pub right_submission_id: i32,
    pub right_match_rate: i32,
    pub lines_matched: i32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = blocks)]
pub struct NewBlock {
    pub match_id: i32,
    pub left_line_from: i32,
    pub left_line_to: i32,
    pub right_line_from: i32,
    pub right_line_to: i32,
}

#[derive(Debug, Queryable)]
pub struct Block {
    pub id: i32,
    pub match_id: i32,
    pub left_line_from: i32,
    pub left_line_to: i32,
    pub right_line_from: i32,
    pub right_line_to: i32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = submissions)]
pub struct NewSubmission {
    pub job_id: i32,
    pub name: String,
    pub code: String,
}

#[derive(Debug, Queryable)]
pub struct Submission {
    pub id: i32,
    pub job_id: i32,
    pub name: String,
    pub code: String,
}
