use rkr_gst::Match;
use std::{
    fs::File,
    io::Read,
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

    let token_template = opts
        .template
        .as_ref()
        .map(|p| jieplag::lang::tokenize(&p).unwrap());
    let token_kind_template: Option<Vec<u8>> =
        token_template.map(|t| t.iter().map(|t| t.kind).collect());

    let initial_search_length = 40;
    let minimum_match_length = 20;
    let matches = rkr_gst::run(
        &token_kind_left,
        &token_kind_right,
        initial_search_length,
        minimum_match_length,
    );

    let left_template_matches = token_kind_template.as_ref().map(|t| {
        rkr_gst::run(
            &token_kind_left,
            t,
            initial_search_length,
            minimum_match_length,
        )
    });
    let right_template_matches = token_kind_template.as_ref().map(|t| {
        rkr_gst::run(
            &token_kind_left,
            t,
            initial_search_length,
            minimum_match_length,
        )
    });

    for (idx, m) in matches.iter().enumerate() {
        // skip if covered
        if let Some(true) = left_template_matches.as_ref().map(|mm| {
            mm.iter().any(|mmm: &Match| {
                mmm.pattern_index <= m.pattern_index + m.length - 1
                    && mmm.pattern_index + mmm.length - 1 >= m.pattern_index
            })
        }) {
            continue;
        }
        if let Some(true) = right_template_matches.as_ref().map(|mm| {
            mm.iter().any(|mmm: &Match| {
                mmm.pattern_index <= m.text_index + m.length - 1
                    && mmm.pattern_index + mmm.length - 1 >= m.text_index
            })
        }) {
            continue;
        }

        let line_from_left = token_left[m.pattern_index].line as usize - 1;
        let line_to_left = token_left[m.pattern_index + m.length - 1].line as usize - 1;
        let line_from_right = token_right[m.text_index].line as usize - 1;
        let line_to_right = token_right[m.text_index + m.length - 1].line as usize - 1;

        println!("Match #{}:", idx + 1);
        println!("Left:");
        println!("{}", lines_left[line_from_left..=line_to_left].join("\n"));
        println!("Right:");
        println!(
            "{}",
            lines_right[line_from_right..=line_to_right].join("\n")
        );
    }

    Ok(())
}
