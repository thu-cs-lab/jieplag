use bitvec::prelude::*;
use rkr_gst::Match;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    /// Path to left source
    #[structopt(short, long)]
    left: PathBuf,

    /// Path to right source
    #[structopt(short, long)]
    right: PathBuf,

    /// Path to template source
    #[structopt(short, long)]
    template: Option<PathBuf>,
}

fn read_file_lines(s: &Path) -> anyhow::Result<Vec<String>> {
    let mut file = File::open(s)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s.lines().map(|l| String::from(l)).collect::<Vec<String>>())
}

fn main() -> anyhow::Result<()> {
    let opts = Args::from_args();
    env_logger::init();

    let token_left = jieplag::lang::tokenize(&opts.left).unwrap();
    let token_kind_left: Vec<u8> = token_left.iter().map(|t| t.kind).collect();
    let lines_left = read_file_lines(&opts.left)?;

    let token_right = jieplag::lang::tokenize(&opts.right).unwrap();
    let token_kind_right: Vec<u8> = token_right.iter().map(|t| t.kind).collect();
    let lines_right = read_file_lines(&opts.right)?;

    let initial_search_length = 40;
    let minimum_match_length = 20;
    let mut matches = rkr_gst::run(
        &token_kind_left,
        &token_kind_right,
        initial_search_length,
        minimum_match_length,
    );

    if let Some(template) = &opts.template {
        let token_template = jieplag::lang::tokenize(&template).unwrap();
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
        let mut filter = |template_matches: &Vec<Match>, left: bool| {
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
                        new_matches.push(Match {
                            pattern_index: m.pattern_index,
                            text_index: m.text_index,
                            length: mmm_from - m_from,
                        });

                        let diff = mmm_to - m_from + 1;
                        new_matches.push(Match {
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

    for is_left in [true, false] {
        let side = if is_left { "left" } else { "right" };
        let mut file = File::create(format!("match-{}.html", side))?;
        writeln!(
            file,
            "<html><head><meta charset=\"UTF-8\"></head><body><pre>"
        )?;

        let mut last_line = 0;

        let colors = ["#FF0000", "#00FF00", "#0000FF", "#00FFFF", "#FF00FF"];
        let mut matches: Vec<(usize, &Match)> = matches.iter().enumerate().collect();

        // sort by line_from
        matches.sort_by_key(|m| {
            if is_left {
                token_left[m.1.pattern_index].line as usize - 1
            } else {
                token_right[m.1.text_index].line as usize - 1
            }
        });
        let token = if is_left { &token_left } else { &token_right };
        let lines = if is_left { &lines_left } else { &lines_right };
        for (idx, m) in matches.iter() {
            let index = if is_left {
                m.pattern_index
            } else {
                m.text_index
            };
            let line_from = token[index].line as usize - 1;
            let line_to = token[index + m.length - 1].line as usize - 1;

            println!("Match #{}:", idx + 1);
            println!("L{}-L{}:", line_from, line_to);
            println!("{}", lines[line_from..=line_to].join("\n"));

            assert!(last_line <= line_from);
            assert!(line_from <= line_to);
            if last_line < line_from {
                writeln!(
                    file,
                    "{}",
                    html_escape::encode_text(&lines[last_line..=(line_from - 1)].join("\n"))
                )?;
            }
            last_line = line_to + 1;

            write!(file, "<font color=\"{}\">", colors[idx % 5])?;
            writeln!(
                file,
                "{}",
                html_escape::encode_text(&lines[line_from..=line_to].join("\n"))
            )?;
            write!(file, "</font>")?;
        }

        if last_line < lines.len() {
            writeln!(
                file,
                "{}",
                html_escape::encode_text(&lines[last_line..].join("\n"))
            )?;
        }

        writeln!(file, "</pre></body></html>")?;
    }

    Ok(())
}
