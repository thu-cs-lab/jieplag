use jieplag::token::Token;
use log::*;
use regex::Regex;
use rkr_gst::Match;
use std::{collections::HashMap, fs::read_dir, path::PathBuf};
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
    #[structopt(short, long, default_value = "0.15")]
    threshold: f32,
}

fn main() -> anyhow::Result<()> {
    let opts = Args::from_args();
    env_logger::init();

    // walk template directory
    info!("Processing template directory");
    let mut template_tokens = HashMap::new();
    for entry in WalkDir::new(&opts.template_directory) {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(&opts.template_directory)?;
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
                    template_tokens.insert(relative_path.to_path_buf(), tokens);
                }
                Err(err) => {
                    warn!("Tokenize {} failed with {}", path.display(), err);
                }
            }
        }
    }

    // walk source directory
    info!("Processing source directory");
    let submissions = read_dir(&opts.source_directory).unwrap();
    // map: file => submission => tokens
    let mut all_tokens: HashMap<PathBuf, HashMap<PathBuf, Vec<Token>>> = HashMap::new();
    for submission in submissions {
        let submission = submission?;
        if !submission.file_type()?.is_dir() {
            continue;
        }
        let submission_directory = opts.source_directory.join(submission.path());
        for entry in WalkDir::new(&submission_directory) {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&submission_directory)?;
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
                        all_tokens
                            .entry(relative_path.to_path_buf())
                            .or_default()
                            .insert(submission.path(), tokens);
                    }
                    Err(err) => {
                        warn!("Tokenize {} failed with {}", path.display(), err);
                    }
                }
            }
        }
    }

    info!("Tokenized {} files in source directory", all_tokens.len());
    for submission in all_tokens.keys() {
        info!("Processing file {}", submission.display());
        let tokens = &all_tokens[submission];
        let keys: Vec<&PathBuf> = all_tokens[submission].keys().collect();

        if !template_tokens.contains_key(submission) {
            continue;
        }

        let template_token = &template_tokens[submission];
        let template_token_kind: Vec<u8> = template_token.iter().map(|t| t.kind).collect();
        let mut match_template: HashMap<usize, Vec<Match>> = HashMap::new();
        for i in 0..keys.len() {
            let token = &tokens[keys[i]];
            let token_kind: Vec<u8> = token.iter().map(|t| t.kind).collect();
            let matches = rkr_gst::run(&token_kind, &template_token_kind, 20, 80);
            match_template.insert(i, matches);
        }

        for left in 0..keys.len() {
            for right in (left + 1)..keys.len() {
                let token_left = &tokens[keys[left]];
                let token_left_kind: Vec<u8> = token_left.iter().map(|t| t.kind).collect();
                let left_match_template = &match_template[&left];
                let token_right = &tokens[keys[right]];
                let token_right_kind: Vec<u8> = token_right.iter().map(|t| t.kind).collect();
                let right_match_template = &match_template[&right];
                let left_match_count: usize = left_match_template.iter().map(|m| m.length).sum();
                let right_match_count: usize = right_match_template.iter().map(|m| m.length).sum();
                // identical to template
                if token_left.len() == left_match_count || token_right.len() == right_match_count {
                    continue;
                }

                let mut matches = rkr_gst::run(&token_left_kind, &token_right_kind, 20, 80);

                let region_intersect = |from: usize, to: usize, matches: &[Match]| {
                    for m in matches.iter() {
                        if to > m.pattern_index && from < m.pattern_index + m.length {
                            return true;
                        }
                    }
                    return false;
                };

                // remove duplicates from template
                matches.retain(|m| {
                    let left_from = m.pattern_index;
                    let left_to = m.pattern_index + m.length;
                    let right_from = m.text_index;
                    let right_to = m.text_index + m.length;
                    !region_intersect(left_from, left_to, left_match_template)
                        && !region_intersect(right_from, right_to, right_match_template)
                });
                let match_count: usize = matches.iter().map(|m| m.length).sum();

                let ratio_left = match_count as f32 / (token_left.len() - left_match_count) as f32;
                let ratio_right =
                    match_count as f32 / (token_right.len() - right_match_count) as f32;
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
    }
    Ok(())
}
