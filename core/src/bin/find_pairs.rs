use clap::Parser;
use core::{
    common::{all_fingerprint, fingerprint, Fingerprint},
    token::Token,
};
use log::*;
use regex::Regex;
use std::{collections::HashMap, fs::read_dir, path::PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
struct Args {
    /// Path to source directory
    #[arg(short, long)]
    source_directory: PathBuf,

    /// Path to template directory
    #[arg(short = 'T', long)]
    template_directory: PathBuf,

    /// Regex patterns for files to include
    #[arg(short, long)]
    include: Vec<Regex>,
}

fn main() -> anyhow::Result<()> {
    let opts = Args::parse();
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
            match core::lang::tokenize(&path) {
                Ok(tokens) => {
                    /*
                    println!("{}:", path.display());
                    for t in &tokens {
                        print!("{}:{}:{:?} ", t.kind, t.line, t.spelling);
                    }
                    println!("");
                    */
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
        let submission_directory = submission.path();
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
                match core::lang::tokenize(&path) {
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
        let keys: Vec<&PathBuf> = all_tokens[submission].keys().collect();

        if !template_tokens.contains_key(submission) {
            continue;
        }

        // https://theory.stanford.edu/~aiken/publications/papers/sigmod03.pdf
        let template_token = &template_tokens[submission];
        let template_fingerprint = all_fingerprint(template_token.iter().map(|t| t.kind), 40);
        let mut local_tokens = vec![];
        let mut local_fingerprints = vec![];
        let mut index: HashMap<u64, Vec<(Fingerprint, usize)>> = HashMap::new();
        for i in 0..keys.len() {
            let token = all_tokens[submission][keys[i]].clone();
            let fingerprint = fingerprint(token.iter().map(|t| t.kind), 40, 80);
            info!(
                "{}: {} tokens, {} fingerprints",
                keys[i].display(),
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
        let mut m = vec![0; keys.len() * keys.len()];
        for hash in index.keys() {
            let v = &index[hash];
            if v.len() > 10 {
                // too common, skip
                continue;
            }

            if v.len() > 5 {
                info!("Found {} entries:", v.len());
                for (f, i) in v {
                    info!(
                        "{} offset {} L{} C{}",
                        keys[*i].display(),
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
                    m[v[i].1 * keys.len() + v[j].1] += 1;
                    m[v[j].1 * keys.len() + v[i].1] += 1;
                }
            }
        }

        let mut sorted_m: Vec<_> = m.iter().enumerate().collect();
        sorted_m.sort_by_key(|(_, val)| **val);
        for (i, matches) in sorted_m.iter().rev().take(40) {
            let left = i % keys.len();
            let right = i / keys.len();
            if left < right {
                // skip duplicatie
                continue;
            }
            let matches = **matches;
            // show info
            info!(
                "Possible plagarism: {} and {}: {} matches",
                keys[left].display(),
                keys[right].display(),
                matches,
            );
        }
    }
    Ok(())
}
