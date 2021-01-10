use adler32::RollingAdler32;
use jieplag::token::Token;
use log::*;
use regex::Regex;
use std::{
    collections::{HashMap, VecDeque},
    fs::{read_dir, File},
    hash::{Hash, Hasher},
    io::Read,
    path::{Path, PathBuf},
};
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
    #[structopt(short, long, default_value = "0.6")]
    threshold: f32,
}

fn read_file_lines(s: &Path) -> anyhow::Result<Vec<String>> {
    let mut file = File::open(s)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s.lines().map(|l| String::from(l)).collect::<Vec<String>>())
}

#[derive(Clone, Copy)]
pub struct Fingerprint {
    pub hash: u64,
    pub offset: usize,
}

fn fingerprint<I>(mut iter: I, noise: usize, guarantee: usize) -> Vec<Fingerprint>
where
    I: Iterator<Item = u8>,
{
    let mut res = vec![];
    // initial rolling `noise`-gram hashes
    let mut items = VecDeque::new();
    let mut hasher = RollingAdler32::new();
    for _ in 0..noise {
        if let Some(e) = iter.next() {
            items.push_back(e);
            hasher.update(e);
        } else {
            // too short
            return res;
        }
    }

    // window of hashes
    let window_size = guarantee - noise + 1;
    let mut hashes = VecDeque::new();
    for _ in 0..window_size {
        hashes.push_back(u64::MAX);
    }

    let mut min_hash_index = 0;
    let mut window_offset = 0;
    while let Some(e) = iter.next() {
        // alder32 is not random enough!
        let mut h = std::collections::hash_map::DefaultHasher::new();
        hasher.hash().hash(&mut h);
        let new_hash = h.finish();

        if new_hash < hashes[min_hash_index] {
            // a new minimum
            min_hash_index = window_size - 1;
            hashes.pop_front();
            hashes.push_back(new_hash);
            res.push(Fingerprint {
                hash: new_hash,
                offset: window_offset,
            });
        } else {
            // update window
            hashes.pop_front();
            hashes.push_back(new_hash);
            if min_hash_index == 0 {
                // rightmost minimum
                for i in (0..window_size).rev() {
                    if hashes[i] < hashes[min_hash_index] {
                        min_hash_index = i;
                    }
                }
                res.push(Fingerprint {
                    hash: new_hash,
                    offset: window_offset - window_size + 1 + min_hash_index,
                });
            } else {
                min_hash_index -= 1;
            }
        }

        // update rolling hash
        hasher.remove(noise, items.pop_front().unwrap());
        items.push_back(e);
        hasher.update(e);
        window_offset += 1;
    }
    res
}

fn all_fingerprint<I>(mut iter: I, noise: usize) -> Vec<Fingerprint>
where
    I: Iterator<Item = u8>,
{
    let mut res = vec![];
    // initial rolling `noise`-gram hashes
    let mut items = VecDeque::new();
    let mut hasher = RollingAdler32::new();
    for _ in 0..noise {
        if let Some(e) = iter.next() {
            items.push_back(e);
            hasher.update(e);
        } else {
            // too short
            return res;
        }
    }

    let mut window_offset = 0;
    while let Some(e) = iter.next() {
        // alder32 is not random enough!
        let mut h = std::collections::hash_map::DefaultHasher::new();
        hasher.hash().hash(&mut h);
        let new_hash = h.finish();

        res.push(Fingerprint {
            hash: new_hash,
            offset: window_offset,
        });

        // update rolling hash
        hasher.remove(noise, items.pop_front().unwrap());
        items.push_back(e);
        hasher.update(e);
        window_offset += 1;
    }
    res
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
        let keys: Vec<&PathBuf> = all_tokens[submission].keys().collect();

        if !template_tokens.contains_key(submission) {
            continue;
        }

        let template_token = &template_tokens[submission];
        let template_fingerprint = all_fingerprint(template_token.iter().map(|t| t.kind), 40);
        let mut local_tokens = vec![];
        let mut local_fingerprints = vec![];
        let mut index: HashMap<u64, Vec<(Fingerprint, usize)>> = HashMap::new();
        for i in 0..keys.len() {
            let token = all_tokens[submission][keys[i]].clone();
            let fingerprint = fingerprint(token.iter().map(|t| t.kind), 40, 80);
            println!(
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
                println!("Found {} entries:", v.len());
                for (f, i) in v {
                    println!(
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
        sorted_m.sort_by_key(|(i, val)| **val);
        for (i, matches) in sorted_m.iter().rev().take(40) {
            let left = i % keys.len();
            let right = i / keys.len();
            if left < right {
                // skip duplicatie
                continue;
            }
            let token_left = &local_tokens[left];
            let token_right = &local_tokens[right];
            let matches = **matches;
            // show info
            info!(
                "Possible plagarism: {} and {}: {} matches",
                keys[left].display(),
                keys[right].display(),
                matches,
            );
        }

        // find large elements

        /*
        for left in 0..keys.len() {
            for right in (left + 1)..keys.len() {
                let token_left = &local_tokens[left];
                let token_left_kind: Vec<u8> = token_left.iter().map(|t| t.kind).collect();
                let token_right = &local_tokens[right];
                let token_right_kind: Vec<u8> = token_right.iter().map(|t| t.kind).collect();
                // too similar to template
                if token_left.len() < 1000 || token_right.len() < 1000 {
                    continue;
                }

                let mut matches = rkr_gst::run(&token_left_kind, &token_right_kind, 40, 20);

                let match_count: usize = matches.iter().map(|m| m.length).sum();

                let ratio_left = match_count as f32 / token_left.len() as f32;
                let ratio_right = match_count as f32 / token_right.len() as f32;
                if ratio_left > opts.threshold && ratio_right > opts.threshold {
                    // convert token matches to line matches
                    let mut line_matches = vec![];
                    for m in &matches {
                        line_matches.push(LineMatch {
                            left_from: token_left[m.pattern_index].line,
                            left_to: token_left[m.pattern_index + m.length - 1].line,
                            right_from: token_right[m.text_index].line,
                            right_to: token_right[m.text_index + m.length - 1].line,
                        });
                    }

                    // merge consecutive matches in line
                    let mut i = 0;
                    while i + 1 < line_matches.len() {
                        if line_matches[i].left_to == line_matches[i + 1].left_from
                            && line_matches[i].right_to == line_matches[i + 1].right_from
                        {
                            line_matches[i].left_to = line_matches[i + 1].left_to;
                            line_matches[i].right_to = line_matches[i + 1].right_to;
                            line_matches.drain(i + 1..i + 2);
                        } else {
                            i = i + 1;
                        }
                    }
                    let left_lines: Vec<String> = read_file_lines(&keys[left].join(&submission))?;
                    let right_lines: Vec<String> = read_file_lines(&keys[right].join(&submission))?;

                    // show info
                    info!(
                        "Possible plagarism: {} and {}: left {} {} right {} {}",
                        keys[left].display(),
                        keys[right].display(),
                        ratio_left,
                        token_left.len(),
                        ratio_right,
                        token_right.len(),
                    );

                    let match_file_name = opts.result_directory.join(format!("match{}", num_match));
                    let mut match_file = File::create(match_file_name)?;
                    writeln!(
                        &mut match_file,
                        "Between {} and {}: {}",
                        keys[left].display(),
                        keys[right].display(),
                        submission.display(),
                    )?;
                    matches.sort_by_key(|m| m.pattern_index);
                    for m in &line_matches {
                        writeln!(
                            &mut match_file,
                            "Left L{}-L{} match Right L{}-L{}:\n{}\n-----------------------------\n{}",
                            m.left_from,
                            m.left_to,
                            m.right_from,
                            m.right_to,
                            left_lines[m.left_from as usize..m.left_to as usize].join("\n"),
                            right_lines[m.right_from as usize..m.right_to as usize].join("\n"),
                        )?;
                    }

                    num_match += 1;
                }
            }
        }
        */
    }
    Ok(())
}
