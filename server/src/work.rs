use anyhow::Context;
use api::def::SubmitRequest;
use core::{
    common::{all_fingerprint, fingerprint, Fingerprint},
    lang::tokenize_str,
    matching::{compute_matching_blocks_from_text, Block},
};

use log::*;
use std::collections::HashMap;

pub struct WorkResult {
    pub req: SubmitRequest,
    pub matches: Vec<Match>,
}

pub struct Match {
    pub left_submission_idx: usize,
    pub left_match_rate: i32,
    pub right_submission_idx: usize,
    pub right_match_rate: i32,
    pub lines_matched: usize,
    pub blocks: Vec<Block>,
}

pub fn work_blocking(req: SubmitRequest) -> anyhow::Result<WorkResult> {
    // tokenize template
    let template_tokens = if let Some(template) = &req.template {
        tokenize_str(template, req.language)?
    } else {
        vec![]
    };

    // tokenize sources
    let mut all_tokens = vec![];
    for submission in &req.submissions {
        all_tokens.push(
            tokenize_str(&submission.code, req.language)
                .with_context(|| submission.name.clone())?,
        );
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
            debug!("Found {} entries:", v.len());
            for (f, i) in v {
                debug!(
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

    let mut matches = vec![];

    // collect highest matches
    let mut sorted_m: Vec<_> = m.iter().enumerate().collect();
    sorted_m.sort_by_key(|(_, val)| **val);
    for (i, num_matches) in sorted_m.iter().rev().take(200) {
        let left = i % all_tokens.len();
        let right = i / all_tokens.len();
        if left <= right {
            // skip duplicate
            continue;
        }
        let num_matches = **num_matches;
        // show debug message
        debug!(
            "Possible plagarism: {} and {}: {} matches",
            req.submissions[left].name, req.submissions[right].name, num_matches,
        );

        let blocks = compute_matching_blocks_from_text(
            &req.submissions[left].code,
            &req.submissions[right].code,
            req.language,
            &req.template,
        )?;

        let mut left_matched_lines = 0;
        let mut right_matched_lines = 0;
        for block in &blocks {
            left_matched_lines += block.left_line_to - block.left_line_from + 1;
            right_matched_lines += block.right_line_to - block.right_line_from + 1;
        }
        let left_lines = req.submissions[left].code.lines().count();
        let right_lines = req.submissions[right].code.lines().count();

        matches.push(Match {
            left_submission_idx: left,
            left_match_rate: (left_matched_lines * 100 / left_lines) as i32,
            right_submission_idx: right,
            right_match_rate: (right_matched_lines * 100 / right_lines) as i32,
            lines_matched: left_matched_lines + right_matched_lines,
            blocks,
        })
    }

    matches.sort_by_key(|m| m.lines_matched);
    matches.reverse();

    Ok(WorkResult { req, matches })
}
