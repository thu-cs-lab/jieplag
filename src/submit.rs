use std::collections::HashMap;

use crate::{
    common::{all_fingerprint, err, fingerprint, Fingerprint},
    lang::{tokenize_str, Language},
    models::User,
    session::{verify, LoginRequest},
    DbPool,
};
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Result};
use diesel::prelude::*;
use log::*;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Submission {
    pub name: String,
    pub code: String,
}

#[derive(Deserialize, Clone)]
pub struct SubmitRequest {
    pub login: Option<LoginRequest>,
    pub language: Language,
    pub template: Option<String>,
    pub submissions: Vec<Submission>,
}

fn work_blocking(req: SubmitRequest) -> anyhow::Result<()> {
    // tokenize template
    let template_tokens = if let Some(template) = &req.template {
        tokenize_str(template, req.language)?
    } else {
        vec![]
    };

    // tokenize sources
    let mut all_tokens = vec![];
    for submission in &req.submissions {
        all_tokens.push(tokenize_str(&submission.code, req.language)?);
    }
    info!("Tokenized {} files in submission", all_tokens.len());

    let template_fingerprint = all_fingerprint(template_tokens.iter().map(|t| t.kind), 40);

    let mut local_tokens = vec![];
    let mut local_fingerprints = vec![];
    let mut index: HashMap<u64, Vec<(Fingerprint, usize)>> = HashMap::new();
    for (i, token) in all_tokens.iter().enumerate() {
        let fingerprint = fingerprint(token.iter().map(|t| t.kind), 40, 80);
        info!(
            "{}: {} tokens, {} fingerprints",
            req.submissions[i].name,
            token.len(),
            fingerprint.len()
        );
        // insert to index: fingerprint => f
        for f in &fingerprint {
            index.entry(f.hash).or_default().push((*f, i));
        }
        local_fingerprints.push(fingerprint);
        local_tokens.push(token);
    }

    // exclude fingerprints in template
    for f in &template_fingerprint {
        index.remove(&f.hash);
    }

    // create two dimensional matrix
    let mut m = vec![0; all_tokens.len() * all_tokens.len()];
    for hash in index.keys() {
        let v = &index[hash];
        if v.len() > 10 {
            // too common, skip
            continue;
        }

        if v.len() > 5 {
            println!("Found {} entries:", v.len());
            for (f, i) in v {
                println!(
                    "{} offset {} L{} C{}",
                    req.submissions[*i].name,
                    f.offset,
                    local_tokens[*i][f.offset].line,
                    local_tokens[*i][f.offset].column,
                );
            }
        }
        // add to matrix
        for i in 0..v.len() {
            for j in (i + 1)..v.len() {
                if v[i].1 == v[j].1 {
                    continue;
                }
                m[v[i].1 * all_tokens.len() + v[j].1] += 1;
                m[v[j].1 * all_tokens.len() + v[i].1] += 1;
            }
        }
    }

    let mut sorted_m: Vec<_> = m.iter().enumerate().collect();
    sorted_m.sort_by_key(|(_, val)| **val);
    for (i, matches) in sorted_m.iter().rev().take(40) {
        let left = i % all_tokens.len();
        let right = i / all_tokens.len();
        if left < right {
            // skip duplicatie
            continue;
        }
        let matches = **matches;
        // show info
        info!(
            "Possible plagarism: {} and {}: {} matches",
            req.submissions[left].name, req.submissions[right].name, matches,
        );
    }

    Ok(())
}

async fn work(req: SubmitRequest, user_id: i32) -> anyhow::Result<()> {
    actix_web::web::block(|| work_blocking(req)).await??;
    Ok(())
}

#[post("/submit")]
pub async fn submit(
    session: Session,
    pool: web::Data<DbPool>,
    body: web::Json<SubmitRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().map_err(err)?;
    use crate::schema::users::dsl;
    if let Some(login) = &body.login {
        if let Ok(user) = dsl::users
            .filter(dsl::user_name.eq(&login.user_name))
            .first::<User>(&mut conn)
        {
            if !verify(&user.salt, &login.password, &user.password) {
                return Ok(HttpResponse::Ok().json(false));
            }
        } else {
            return Ok(HttpResponse::Ok().json(false));
        }
    } else {
        if let Some(user_id) = session.get::<i32>("id")? {
            work((*body).clone(), user_id).await.map_err(err)?;
            return Ok(HttpResponse::Ok().json(false));
        }
    }
    Ok(HttpResponse::Ok().json(false))
}
