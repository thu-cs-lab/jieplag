use crate::token::Token;
use actix_web::{error::ErrorInternalServerError, Error};
use adler32::RollingAdler32;
use anyhow::anyhow;
use log::*;
use std::{
    collections::VecDeque,
    fmt::Display,
    hash::{Hash, Hasher},
};
use uuid::Uuid;

#[derive(Copy, Clone)]
pub struct LineMatch {
    pub left_from: u32,
    pub left_to: u32,
    pub right_from: u32,
    pub right_to: u32,
}

pub fn find_matches(left: &[Token], right: &[Token]) -> Vec<LineMatch> {
    let mut line_matches = vec![];
    let left_kind: Vec<u8> = left.iter().map(|t| t.kind).collect();
    let right_kind: Vec<u8> = right.iter().map(|t| t.kind).collect();
    let mut matches = rkr_gst::run(&left_kind, &right_kind, 40, 20);
    matches.sort_by_key(|m| m.pattern_index);

    for m in &matches {
        line_matches.push(LineMatch {
            left_from: left[m.pattern_index].line,
            left_to: left[m.pattern_index + m.length - 1].line,
            right_from: right[m.text_index].line,
            right_to: right[m.text_index + m.length - 1].line,
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
    line_matches
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fingerprint {
    pub hash: u64,
    pub offset: usize,
}

// https://theory.stanford.edu/~aiken/publications/papers/sigmod03.pdf
// void winnow(int w)
pub fn fingerprint<I>(mut iter: I, noise: usize, guarantee: usize) -> Vec<Fingerprint>
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
                    hash: hashes[min_hash_index],
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

pub fn all_fingerprint<I>(mut iter: I, noise: usize) -> Vec<Fingerprint>
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

pub fn generate_uuid() -> String {
    let uuid = Uuid::new_v4();
    uuid.simple()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
}

#[track_caller]
pub fn err<T: Display>(err: T) -> Error {
    let error_token = generate_uuid();
    let location = std::panic::Location::caller();
    error!("Error {} at {}: {}", error_token, location, err);
    ErrorInternalServerError(anyhow!(format!(
        "Please contact admin with error token {}",
        error_token
    )))
}

#[cfg(test)]
mod tests {
    use super::all_fingerprint;
    use super::fingerprint;

    #[test]
    fn test_all_fingerprint() {
        // example taken from paper
        let text = "adorunrunrunadorunrun";
        let fingerprints = all_fingerprint(text.bytes(), 5);
        eprintln!("{:?}", fingerprints);

        // adoru @ 0, 12
        assert_eq!(fingerprints[0].hash, fingerprints[12].hash);

        // dorun @ 1, 13
        assert_eq!(fingerprints[1].hash, fingerprints[13].hash);

        // runru @ 3, 6, 15
        assert_eq!(fingerprints[3].hash, fingerprints[6].hash);
        assert_eq!(fingerprints[3].hash, fingerprints[15].hash);
    }

    #[test]
    fn test_fingerprint() {
        // example taken from paper
        let text = "adorunrunrunadorunrun";
        let all_fingerprints = all_fingerprint(text.bytes(), 5);
        eprintln!("{:?}", all_fingerprints);

        // windows size of hashes = 2
        let fingerprints = fingerprint(text.bytes(), 5, 6);
        eprintln!("{:?}", fingerprints);

        // check if subset
        for f in &fingerprints {
            assert!(all_fingerprints.contains(f), "{:?} not found", f);
        }
    }
}

pub fn gen_svg(color: &str, ratio: i32) -> String {
    format!(
        r#"<svg width="60" height="12">
    <rect width="60" height="12" style="fill:rgba(0,0,0,0);stroke-width:4;stroke:{}"></rect>
    <rect width="{}" height="12" style="fill:{};"></rect>
</svg>"#,
        color,
        60 * ratio / 100,
        color
    )
}
