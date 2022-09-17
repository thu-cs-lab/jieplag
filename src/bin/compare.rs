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
    #[structopt(short = "T", long)]
    right: PathBuf,
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
    let token_left_kind: Vec<u8> = token_left.iter().map(|t| t.kind).collect();
    let lines_left = read_file_lines(&opts.left)?;
    let token_right = jieplag::lang::tokenize(&opts.right).unwrap();
    let token_right_kind: Vec<u8> = token_right.iter().map(|t| t.kind).collect();
    let lines_right = read_file_lines(&opts.right)?;

    let matches = rkr_gst::run(&token_left_kind, &token_right_kind, 40, 20);

    for (idx, m) in matches.iter().enumerate() {
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
