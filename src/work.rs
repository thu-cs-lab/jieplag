use crate::{
    common::{all_fingerprint, fingerprint, Fingerprint},
    lang::{tokenize_str, Language},
    submit::SubmitRequest,
};
use bitvec::prelude::*;
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

pub struct Block {
    // 0-based
    pub left_line_from: usize,
    pub left_line_to: usize,
    pub right_line_from: usize,
    pub right_line_to: usize,
}

/// Compute matching blocks via RKR-GST algorithm
pub fn compute_matching_blocks(
    left: &str,
    right: &str,
    language: Language,
    template: &Option<String>,
) -> anyhow::Result<Vec<Block>> {
    let token_left = crate::lang::tokenize_str(left, language)?;
    let token_kind_left: Vec<u8> = token_left.iter().map(|t| t.kind).collect();
    let lines_left: Vec<&str> = left.lines().collect();

    let token_right = crate::lang::tokenize_str(&right, language).unwrap();
    let token_kind_right: Vec<u8> = token_right.iter().map(|t| t.kind).collect();
    let lines_right: Vec<&str> = right.lines().collect();

    let initial_search_length = 40;
    let minimum_match_length = 20;
    let mut matches = rkr_gst::run(
        &token_kind_left,
        &token_kind_right,
        initial_search_length,
        minimum_match_length,
    );

    if let Some(template) = &template {
        let token_template = crate::lang::tokenize_str(template, language).unwrap();
        let token_kind_template: Vec<u8> = token_template.iter().map(|t| t.kind).collect();

        let left_template_matches = rkr_gst::run(
            &token_kind_left,
            &token_kind_template,
            initial_search_length,
            minimum_match_length,
        );
        let right_template_matches = rkr_gst::run(
            &token_kind_right,
            &token_kind_template,
            initial_search_length,
            minimum_match_length,
        );

        // filter covered parts
        let mut filter = |template_matches: &Vec<rkr_gst::Match>, left: bool| {
            for mmm in template_matches {
                let mmm_from = mmm.pattern_index;
                let mmm_to = mmm.pattern_index + mmm.length - 1;

                let mut new_matches = vec![];
                for m in matches.iter() {
                    let mut m = m.clone();
                    let m_from = if left { m.pattern_index } else { m.text_index };
                    let m_to = m_from + m.length - 1;

                    if mmm_from <= m_from && m_to <= mmm_to {
                        // fully covered
                        continue;
                    } else if mmm_from <= m_from && m_from <= mmm_to {
                        // move head
                        let diff = mmm_to - m_from + 1;
                        m.pattern_index += diff;
                        m.text_index += diff;
                        m.length -= diff;
                        new_matches.push(m);
                    } else if mmm_from <= m_to && m_to <= mmm_to {
                        // move tail
                        let diff = m_to - mmm_from + 1;
                        m.length -= diff;
                        new_matches.push(m);
                    } else if m_from <= mmm_from && mmm_to <= m_to {
                        // split into two
                        new_matches.push(rkr_gst::Match {
                            pattern_index: m.pattern_index,
                            text_index: m.text_index,
                            length: mmm_from - m_from,
                        });

                        let diff = mmm_to - m_from + 1;
                        new_matches.push(rkr_gst::Match {
                            pattern_index: m.pattern_index + diff,
                            text_index: m.text_index + diff,
                            length: m.length - diff,
                        });
                    } else {
                        // unchanged
                        new_matches.push(m);
                    }
                }
                matches = new_matches;
            }
        };
        filter(&left_template_matches, true);
        filter(&right_template_matches, false);
    }

    // ensure matches are on distinct lines
    let mut bitvec_left = bitvec![0; lines_left.len()];
    let mut bitvec_right = bitvec![0; lines_right.len()];
    let mut i = 0;
    while i < matches.len() {
        let m = matches[i];
        let line_from_left = token_left[m.pattern_index].line as usize - 1;
        let line_to_left = token_left[m.pattern_index + m.length - 1].line as usize - 1;
        let line_from_right = token_right[m.text_index].line as usize - 1;
        let line_to_right = token_right[m.text_index + m.length - 1].line as usize - 1;
        if !bitvec_left[line_from_left]
            && !bitvec_left[line_to_left]
            && !bitvec_right[line_from_right]
            && !bitvec_right[line_to_right]
        {
            // safe
            for i in line_from_left..=line_to_left {
                bitvec_left.set(i, true);
            }
            for i in line_from_right..=line_to_right {
                bitvec_right.set(i, true);
            }
            i += 1;
        } else {
            matches.remove(i);
        }
    }

    let mut res = vec![];

    for (idx, m) in matches.iter().enumerate() {
        let line_from_left = token_left[m.pattern_index].line as usize - 1;
        let line_to_left = token_left[m.pattern_index + m.length - 1].line as usize - 1;
        let line_from_right = token_right[m.text_index].line as usize - 1;
        let line_to_right = token_right[m.text_index + m.length - 1].line as usize - 1;

        res.push(Block {
            left_line_from: line_from_left,
            left_line_to: line_to_left,
            right_line_from: line_from_right,
            right_line_to: line_to_right,
        });

        debug!("Match #{}:", idx + 1);
        debug!("Left L{}-L{}", line_from_left, line_to_left);
        debug!("Right L{}-L{}", line_from_right, line_to_right);
    }
    Ok(res)
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
    for (i, num_matches) in sorted_m.iter().rev().take(40) {
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

        let blocks = compute_matching_blocks(
            &req.submissions[left].code,
            &req.submissions[right].code,
            req.language,
            &req.template,
        )?;

        matches.push(Match {
            left_submission_idx: left,
            left_match_rate: 0,
            right_submission_idx: right,
            right_match_rate: 0,
            lines_matched: 0,
            blocks,
        })
    }

    Ok(WorkResult { req, matches })
}
