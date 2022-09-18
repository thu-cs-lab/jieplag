use crate::{
    common::err,
    models::{Block, Job, Match, Submission},
    DbPool,
};
use actix_web::{get, http::header, web, HttpResponse, Result};
use diesel::prelude::*;

#[get("/results/{slug}/{match_id}/{frame}")]
pub async fn match_inner(
    pool: web::Data<DbPool>,
    path: web::Path<(String, i64, String)>,
) -> Result<HttpResponse> {
    let (slug, match_id, frame) = path.into_inner();
    let (is_top, is_left) = match frame.as_str() {
        "left" => (false, true),
        "right" => (false, false),
        "top" => (true, false),
        _ => {
            return Ok(HttpResponse::NotFound().json(false));
        }
    };

    let mut conn = pool.get().map_err(err)?;
    let job = crate::schema::jobs::dsl::jobs
        .filter(crate::schema::jobs::dsl::slug.eq(&*slug))
        .first::<Job>(&mut conn)
        .map_err(err)?;
    let m = crate::schema::matches::dsl::matches
        .filter(crate::schema::matches::dsl::job_id.eq(job.id))
        .offset(match_id)
        .first::<Match>(&mut conn)
        .map_err(err)?;
    let blocks = crate::schema::blocks::dsl::blocks
        .filter(crate::schema::blocks::dsl::match_id.eq(m.id))
        .load::<Block>(&mut conn)
        .map_err(err)?;
    let mut res;
    if is_top {
        res = "<html><head><meta charset=\"UTF-8\"></head>".to_string();
        res += "<body><center><table border=\"1\" cellspacing=\"0\" bgcolor=\"#d0d0d0\">";
        res += "<tbody>";
        // add title

        res += "<tr>";
        let left_s = crate::schema::submissions::dsl::submissions
            .filter(crate::schema::submissions::dsl::id.eq(m.left_submission_id))
            .first::<Submission>(&mut conn)
            .map_err(err)?;
        res += &format!("<th>{} ({}%)</th>", left_s.name, m.left_match_rate);
        res += "<th><img src=\"http://moss.stanford.edu/bitmaps/tm_0_30.gif\" border=\"0\" align=\"left\"></th>";
        let right_s = crate::schema::submissions::dsl::submissions
            .filter(crate::schema::submissions::dsl::id.eq(m.right_submission_id))
            .first::<Submission>(&mut conn)
            .map_err(err)?;
        res += &format!("<th>{} ({}%)</th>", right_s.name, m.right_match_rate);
        res += "<th><img src=\"http://moss.stanford.edu/bitmaps/tm_0_30.gif\" border=\"0\" align=\"left\"></th>";
        res += "<th> </th>";
        res += "</tr>";

        let left_lines = left_s.code.lines().count();
        let right_lines = right_s.code.lines().count();

        // add index to blocks before sorting
        // so that index remains sync-ed in top, left & right panels
        let mut blocks: Vec<(usize, Block)> = blocks.into_iter().enumerate().collect();

        // sort by line counts
        blocks.sort_by_key(|b| {
            b.1.left_line_to - b.1.left_line_from + b.1.right_line_to - b.1.right_line_from
        });
        blocks.reverse();

        for (idx, block) in blocks.iter() {
            res += "<tr>";
            res += &format!(
                "<td><a href=\"./left#{}\" target=\"left\">{}-{}</td>",
                block.left_line_from, block.left_line_from, block.left_line_to
            );
            let left_ratio =
                (block.left_line_to - block.left_line_from + 1) * 100 / left_lines as i32;
            res += &format!("<td><img src=\"http://moss.stanford.edu/bitmaps/tm_{}_{}.gif\" border=\"0\" align=\"left\"></td>", idx % 5, left_ratio);
            res += &format!(
                "<td><a href=\"./right#{}\" target=\"right\">{}-{}</td>",
                block.right_line_from, block.right_line_from, block.right_line_to
            );
            let right_ratio =
                (block.right_line_to - block.right_line_from + 1) * 100 / right_lines as i32;
            res += &format!("<td><img src=\"http://moss.stanford.edu/bitmaps/tm_{}_{}.gif\" border=\"0\" align=\"right\"></td>", idx % 5, right_ratio);
            res += "<td> </td>";
            res += "</tr>";
        }

        res += "</tbody></table></center></body></html>";
    } else {
        let submission_id = if is_left {
            m.left_submission_id
        } else {
            m.right_submission_id
        };
        let s = crate::schema::submissions::dsl::submissions
            .filter(crate::schema::submissions::dsl::id.eq(submission_id))
            .first::<Submission>(&mut conn)
            .map_err(err)?;
        let lines: Vec<&str> = s.code.lines().collect();

        res = "<html><head><meta charset=\"UTF-8\"></head><body><pre>".to_string();
        let mut last_line = 0;
        let colors = ["#FF0000", "#00FF00", "#0000FF", "#00FFFF", "#FF00FF"];

        // add index to blocks before sorting
        // so that index remains sync-ed in left & right panels
        let mut blocks: Vec<(usize, Block)> = blocks.into_iter().enumerate().collect();

        // sort by line_from
        blocks.sort_by_key(|b| {
            if is_left {
                b.1.left_line_from
            } else {
                b.1.right_line_from
            }
        });

        for (idx, b) in blocks.iter() {
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

            assert!(last_line <= line_from);
            assert!(line_from <= line_to);
            if last_line < line_from {
                res += &html_escape::encode_text(&lines[last_line..=(line_from - 1)].join("\n"))
                    .to_string();
                res += "\n";
            }
            last_line = line_to + 1;

            // add link to jump to opposite side
            res += &format!("<a name=\"{}\">", line_from);
            res += &format!("<font color=\"{}\">", colors[idx % 5]);
            res += &format!(
                "<a href=\"{}#{}\" target=\"{}\">",
                opposite_side, opposite_line_from, opposite_side
            );
            res += &format!("<img src=\"http://moss.stanford.edu/bitmaps/tm_{}_1.gif\" alt=\"other\" border=\"0\" align=\"left\"></a>", idx % 5);
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
            res += &html_escape::encode_text(&lines[last_line..].join("\n")).to_string();
            res += "\n";
        }

        res += "</pre></body></html>";
    }
    return Ok(HttpResponse::Ok()
        .append_header(header::ContentType::html())
        .body(res));
}

#[get("/results/{slug}/{match_id}/")]
pub async fn match_two_columns(_path: web::Path<(String, i64)>) -> Result<HttpResponse> {
    let res = format!(
        r#"
<html>
	<head>
	</head>
    <frameset rows="150,*">
        <frameset cols="1000,*">
            <frame src="./top" name="top" frameborder="0"></frame>
        </frameset>
        <frameset cols="50%,50%">
            <frame src="./left" name="left"></frame>
            <frame src="./right" name="right"></frame>
        </frameset>
    </frameset>
</html>
    "#
    );

    return Ok(HttpResponse::Ok()
        .append_header(header::ContentType::html())
        .body(res));
}
