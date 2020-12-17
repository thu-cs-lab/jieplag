use log::*;
use regex::Regex;
use std::{collections::HashMap, path::PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt)]
struct Args {
    /// Path to source directory
    #[structopt(short, long)]
    source_directory: PathBuf,

    /// Path to template directory
    #[structopt(short = "T", long)]
    template_directory: PathBuf,

    /// Path to result directory
    #[structopt(short, long)]
    result_directory: PathBuf,

    /// Regex patterns for files to include
    #[structopt(short, long)]
    include: Vec<Regex>,

    /// Threshold of similarity
    #[structopt(short, long, default_value = "0.5")]
    threshold: f32,
}

fn main() -> anyhow::Result<()> {
    let opts = Args::from_args();
    env_logger::init();

    // walk source directory
    let mut all_tokens = HashMap::new();
    for entry in WalkDir::new(&opts.source_directory) {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(&opts.source_directory)?;
        let mut include = false;
        for pattern in &opts.include {
            if pattern.is_match(&relative_path.display().to_string()) {
                include = true;
                break;
            }
        }
        if include {
            match jieplag::lang::tokenize(&path) {
                Ok(tokens) => {
                    all_tokens.insert(relative_path.to_path_buf(), tokens);
                }
                Err(err) => {
                    warn!("Tokenize {} failed with {}", path.display(), err);
                }
            }
        }
    }

    info!("Tokenized {} files in source directory", all_tokens.len());
    let keys: Vec<&PathBuf> = all_tokens.keys().collect();
    for left in 0..keys.len() {
        for right in (left + 1)..keys.len() {
            let token_left = &all_tokens[keys[left]];
            let token_left_kind: Vec<u8> = token_left.iter().map(|t| t.kind).collect();
            let token_right = &all_tokens[keys[right]];
            let token_right_kind: Vec<u8> = token_right.iter().map(|t| t.kind).collect();
            let matches = rkr_gst::run(&token_left_kind, &token_right_kind, 20, 80);
            let match_count: usize = matches.iter().map(|m| m.length).sum();
            let ratio_left = match_count as f32 / token_left.len() as f32;
            let ratio_right = match_count as f32 / token_right.len() as f32;
            if ratio_left > opts.threshold || ratio_right > opts.threshold {
                info!(
                    "Possible plagarism: {} and {}: {} {}",
                    keys[left].display(),
                    keys[right].display(),
                    ratio_left,
                    ratio_right
                );
            }
        }
    }
    Ok(())
}
