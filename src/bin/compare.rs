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

    let mut file_left = File::create("match-left.html")?;
    writeln!(
        file_left,
        "<html><head><meta charset=\"UTF-8\"></head><body><pre>"
    )?;
    let mut file_right = File::create("match-right.html")?;
    writeln!(
        file_right,
        "<html><head><meta charset=\"UTF-8\"></head><body><pre>"
    )?;

    let mut last_line_left = 0;
    let mut last_line_right = 0;
    let colors = ["#FF0000", "#00FF00", "#0000FF"];
    for (idx, m) in matches.iter().enumerate() {
        let line_from_left = token_left[m.pattern_index].line as usize - 1;
        let line_to_left = token_left[m.pattern_index + m.length - 1].line as usize - 1;
        let line_from_right = token_right[m.text_index].line as usize - 1;
        let line_to_right = token_right[m.text_index + m.length - 1].line as usize - 1;

        if last_line_left < line_from_left {
            writeln!(
                file_left,
                "{}",
                html_escape::encode_text(
                    &lines_left[last_line_left..=(line_from_left - 1)].join("\n")
                )
            )?;
            last_line_left = line_to_left + 1;
        }
        writeln!(file_left, "<font color=\"{}\">", colors[idx % 3])?;
        writeln!(
            file_left,
            "{}",
            html_escape::encode_text(&lines_left[line_from_left..=line_to_left].join("\n"))
        )?;
        writeln!(file_left, "</font>")?;

        if last_line_right < line_from_right {
            writeln!(
                file_right,
                "{}",
                html_escape::encode_text(
                    &lines_right[last_line_right..=(line_from_right - 1)].join("\n")
                )
            )?;
            last_line_right = line_to_right + 1;
        }
        writeln!(file_right, "<font color=\"{}\">", colors[idx % 3])?;
        writeln!(
            file_right,
            "{}",
            html_escape::encode_text(&lines_right[line_from_right..=line_to_right].join("\n"))
        )?;
        writeln!(file_right, "</font>")?;

        println!("Match #{}:", idx + 1);
        println!("Left L{}-L{}:", line_from_left, line_to_left);
        println!("{}", lines_left[line_from_left..=line_to_left].join("\n"));
        println!("Right L{}-L{}:", line_from_right, line_to_right);
        println!(
            "{}",
            lines_right[line_from_right..=line_to_right].join("\n")
        );
    }

    writeln!(file_left, "</pre></body></html>")?;

    Ok(())
}
