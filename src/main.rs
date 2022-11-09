mod candidate;
mod street_matcher;
mod text_matcher;

use text_matcher_rs::TextMatcher;

use crate::street_matcher::{MatchedStreet, StreetMatcher};
use std::panic;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

const CHUNK_SIZE: usize = 1000;

#[inline]
fn join_to_row(index: &str, street: &str, place: &str, mstreet: MatchedStreet) -> String {
    [
        index,
        street,
        place,
        mstreet.street.unwrap_or_default().as_str(),
        mstreet
            .file_found
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default(),
    ]
    .join("\t")
}

fn main() {
    let reader = BufReader::new(File::open("./test_data/real_data.csv").expect("file exists"));
    let street_matcher = StreetMatcher::default();
    let mut file_out = File::options()
        .create(true)
        .write(true)
        .append(true)
        .open("./test_data/real_data_output.csv")
        .unwrap();
    let _: Vec<_> = reader
        .lines()
        .flatten()
        .collect::<Vec<String>>()
        .chunks(CHUNK_SIZE)
        .map(|chunk| {
            let lines = chunk
                .iter()
                .flat_map(|line| {
                    if let [index, street, place] =
                        line.split('\t').take(3).collect::<Vec<&str>>()[..]
                    {
                        panic::catch_unwind(|| street_matcher.match_by_place(street, Some(place)))
                            .ok()
                            .map(|mat| join_to_row(index, street, place, mat))
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
            if lines.is_empty() {
                println!("Lines are empty, skipping...");
                return;
            }
            match writeln!(file_out, "{}", lines) {
                Ok(_) => {
                    println!("Successfully written chunk to file");
                }
                Err(err) => {
                    eprintln!("Couldn't write to file: {}", err);
                }
            }
        })
        .collect();
}
