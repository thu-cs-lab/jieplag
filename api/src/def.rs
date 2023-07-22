use core::lang::Language;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginRequest {
    pub user_name: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Submission {
    pub name: String,
    pub code: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubmitRequest {
    pub login: Option<LoginRequest>,
    pub language: Language,
    pub template: Option<String>,
    pub submissions: Vec<Submission>,
}
