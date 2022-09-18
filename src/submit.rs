use crate::{
    common::{err, generate_uuid},
    lang::Language,
    models::{NewBlock, NewJob, NewMatch, NewSubmission, User},
    session::{verify, LoginRequest},
    work::work_blocking,
    DbConnection, DbPool,
};
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Result};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
};
use log::*;
use serde::{Deserialize, Serialize};

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

async fn work(
    mut conn: PooledConnection<ConnectionManager<DbConnection>>,
    req: SubmitRequest,
    user_id: i32,
) -> anyhow::Result<String> {
    let work_req = req.clone();
    let work = actix_web::web::block(move || work_blocking(work_req)).await??;
    let slug = conn.transaction::<_, diesel::result::Error, _>(move |conn| {
        // create new job
        let slug = generate_uuid();
        let new_job = NewJob {
            creator_user_id: user_id,
            slug: slug.clone(),
        };
        let job_ids: Vec<i32> = diesel::insert_into(crate::schema::jobs::table)
            .values(new_job)
            .returning(crate::schema::jobs::dsl::id)
            .get_results(conn)?;
        let job_id = job_ids[0];

        // insert submissions
        let new_submissions: Vec<NewSubmission> = req
            .submissions
            .iter()
            .map(|s| NewSubmission {
                job_id,
                name: s.name.clone(),
                code: s.code.clone(),
            })
            .collect();
        let submission_ids: Vec<i32> = diesel::insert_into(crate::schema::submissions::table)
            .values(new_submissions)
            .returning(crate::schema::submissions::dsl::id)
            .get_results(conn)?;

        // insert matches
        let new_matches: Vec<NewMatch> = work
            .matches
            .iter()
            .map(|m| NewMatch {
                job_id,
                left_submission_id: submission_ids[m.left_submission_idx],
                left_match_rate: m.left_match_rate,
                right_submission_id: submission_ids[m.right_submission_idx],
                right_match_rate: m.right_match_rate,
                lines_matched: m.lines_matched as i32,
            })
            .collect();
        let match_ids: Vec<i32> = diesel::insert_into(crate::schema::matches::table)
            .values(new_matches)
            .returning(crate::schema::matches::dsl::id)
            .get_results(conn)?;

        for (match_id, m) in match_ids.iter().zip(work.matches.iter()) {
            // insert blocks
            let new_blocks: Vec<NewBlock> = m
                .blocks
                .iter()
                .map(|b| NewBlock {
                    match_id: *match_id,
                    left_line_from: b.left_line_from as i32,
                    left_line_to: b.left_line_to as i32,
                    right_line_from: b.right_line_from as i32,
                    right_line_to: b.right_line_to as i32,
                })
                .collect();
            diesel::insert_into(crate::schema::blocks::table)
                .values(new_blocks)
                .execute(conn)?;
        }
        Ok(slug)
    })?;
    Ok(slug)
}

#[post("/submit")]
pub async fn submit(
    session: Session,
    pool: web::Data<DbPool>,
    body: web::Json<SubmitRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool.get().map_err(err)?;
    use crate::schema::users::dsl;
    let user_id;
    if let Some(login) = &body.login {
        if let Ok(user) = dsl::users
            .filter(dsl::user_name.eq(&login.user_name))
            .first::<User>(&mut conn)
        {
            if verify(&user.salt, &login.password, &user.password) {
                user_id = user.id;
            } else {
                return Ok(HttpResponse::Ok().json(false));
            }
        } else {
            return Ok(HttpResponse::Ok().json(false));
        }
    } else {
        if let Some(id) = session.get::<i32>("id")? {
            user_id = id;
        } else {
            return Ok(HttpResponse::Ok().json(false));
        }
    }
    info!("Got submission from {}", user_id);
    let slug = work(conn, (*body).clone(), user_id).await.map_err(err)?;
    return Ok(HttpResponse::Ok().json(slug));
}
