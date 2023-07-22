use crate::lang::Language;
use crate::token::Token;

use bitvec::bitvec;
use log::*;
use rkr_gst::Match;

pub fn compute_matches_from_token(
    token_left: &[Token],
    token_kind_left: &[u8],
    lines_left: &[&str],
    token_right: &[Token],
    token_kind_right: &[u8],
    lines_right: &[&str],
    template_kind: Option<&[u8]>,
) -> Vec<Match> {
    let initial_search_length = 40;
    let minimum_match_length = 20;
    let mut matches = rkr_gst::run(
        &token_kind_left,
        &token_kind_right,
        initial_search_length,
        minimum_match_length,
    );

    if let Some(token_kind_template) = template_kind {
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

    matches
}

pub struct Block {
    // 0-based
    pub left_line_from: usize,
    pub left_line_to: usize,
    pub right_line_from: usize,
    pub right_line_to: usize,
}

/// Compute matching blocks via RKR-GST algorithm
pub fn compute_matching_blocks_from_text(
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

    let template_kind: Option<Vec<u8>> = template.as_ref().map(|t| {
        let token_template = crate::lang::tokenize_str(t, language).unwrap();
        token_template.iter().map(|t| t.kind).collect()
    });

    let matches = compute_matches_from_token(
        &token_left,
        &token_kind_left,
        &lines_left,
        &token_right,
        &token_kind_right,
        &lines_right,
        template_kind.as_ref().map(|v| v.as_slice()),
    );

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
