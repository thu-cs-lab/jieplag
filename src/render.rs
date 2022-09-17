use crate::{
    common::err,
    models::{Block, Job, Match, Submission},
    DbPool,
};
use actix_web::{get, http::header, web, HttpResponse, Result};
use diesel::prelude::*;

#[get("/results/{slug}/{match_id}/{left_right}")]
pub async fn match_inner(
    pool: web::Data<DbPool>,
    path: web::Path<(String, i64, String)>,
) -> Result<HttpResponse> {
    let (slug, match_id, left_right) = path.into_inner();
    let is_left = match left_right.as_str() {
        "left" => true,
        "right" => false,
        _ => {
            return Ok(HttpResponse::NotFound().json(false));
        }
    };

    let mut conn = pool.get().map_err(err)?;
    if let Ok(job) = crate::schema::jobs::dsl::jobs
        .filter(crate::schema::jobs::dsl::slug.eq(&*slug))
        .first::<Job>(&mut conn)
    {
        if let Ok(m) = crate::schema::matches::dsl::matches
            .filter(crate::schema::matches::dsl::job_id.eq(job.id))
            .offset(match_id)
            .first::<Match>(&mut conn)
        {
            let submission_id = if is_left {
                m.left_submission_id
            } else {
                m.right_submission_id
            };
            if let Ok(s) = crate::schema::submissions::dsl::submissions
                .filter(crate::schema::submissions::dsl::id.eq(submission_id))
                .first::<Submission>(&mut conn)
            {
                let lines: Vec<&str> = s.code.lines().collect();
                if let Ok(mut blocks) = crate::schema::blocks::dsl::blocks
                    .filter(crate::schema::blocks::dsl::match_id.eq(m.id))
                    .load::<Block>(&mut conn)
                {
                    let mut res =
                        "<html><head><meta charset=\"UTF-8\"></head><body><pre>".to_string();
                    let mut last_line = 0;
                    let colors = ["#FF0000", "#00FF00", "#0000FF"];

                    // sort by line_from
                    blocks.sort_by_key(|b| {
                        if is_left {
                            b.left_line_from
                        } else {
                            b.right_line_from
                        }
                    });

                    for (idx, b) in blocks.iter().enumerate() {
                        let line_from = if is_left {
                            b.left_line_from
                        } else {
                            b.right_line_from
                        } as usize;

                        let opposite_line_from = if is_left {
                            b.right_line_from
                        } else {
                            b.left_line_from
                        } as usize;
                        let opposite_side = if is_left { "right" } else { "left" };

                        let line_to = if is_left {
                            b.left_line_to
                        } else {
                            b.right_line_to
                        } as usize;

                        if last_line < line_from {
                            res += &html_escape::encode_text(
                                &lines[last_line..=(line_from - 1)].join("\n"),
                            )
                            .to_string();
                            res += "\n";
                            last_line = line_to + 1;
                        }

                        // add link to jump to opposite side
                        res += &format!("<a name=\"{}\">", line_from);
                        res += &format!("<font color=\"{}\">", colors[idx % 3]);
                        res += &format!(
                            "<a href=\"{}#{}\" target=\"{}\">",
                            opposite_side, opposite_line_from, opposite_side
                        );
                        res += &format!("<img src=\"http://moss.stanford.edu/bitmaps/tm_3_2.gif\" alt=\"other\" border=\"0\" align=\"left\"></a>");
                        res += "\n";
                        res += &format!(
                            "{}",
                            html_escape::encode_text(&lines[line_from..=line_to].join("\n"))
                        );
                        res += "\n";
                        res += "</font>";
                    }

                    // the rest
                    if last_line < lines.len() {
                        res +=
                            &html_escape::encode_text(&lines[last_line..].join("\n")).to_string();
                        res += "\n";
                    }

                    res += "</pre></body></html>";
                    return Ok(HttpResponse::Ok()
                        .append_header(header::ContentType::html())
                        .body(res));
                }
            }
        }
    }
    Ok(HttpResponse::NotFound().json(false))
}

#[get("/results/{slug}/{match_id}/")]
pub async fn match_two_columns(path: web::Path<(String, i64)>) -> Result<HttpResponse> {
    let (_slug, match_id) = path.into_inner();

    let res = format!(
        r#"
<html>
	<head>
	</head>
	<frameset cols="50%,50%">
		<frame src="./left" name="left"></frame>
		<frame src="./right" name="right"></frame>
	</frameset>
</html>
    "#
    );

    return Ok(HttpResponse::Ok()
        .append_header(header::ContentType::html())
        .body(res));
}
