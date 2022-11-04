use crate::{candidate::Candidate, text_matcher::TextMatcher};

use regex::Regex;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

const SENSITIVITY: f64 = 0.6;
const FILE_SENSITIVITY: f64 = 0.87;
const PLACE_SEARCH_SENSITIVITY: f64 = 0.7;
const KEEP: usize = 50;
const PATH_TO_PLACES: &str = "./test_data/places.txt";
const PATH_TO_PLZS_DIR: &str = "./test_data/plzs/";
const PATH_TO_PLACES_DIR: &str = "./test_data/places/";

#[derive(Debug)]
pub struct MatchedStreet {
    pub street: Option<String>,
    pub file_found: Option<PathBuf>,
}

fn does_start_with_number(street: &str) -> bool {
    Regex::new(r"^\d+,?\s.+").unwrap().is_match(street)
        || Regex::new(r"^\w\d+,?\s").unwrap().is_match(street)
}

fn clean_street(street: &str) -> String {
    let mut street = String::from(street.trim()).to_lowercase();
    // Matches: '76 chemin des clos' or 'a4 résidence du golf'
    if does_start_with_number(&street) {
        let mut parts = street.split_whitespace();
        let number = parts
            .next()
            .expect("the string format is correct due to regex");
        street = format!("{} {}", parts.collect::<Vec<&str>>().join(" "), number);
    }
    // TODO: add punctuation removing
    // Matches: eisfeldstrasse 21/23, milchstrasse 2-10a, milchstrasse 2,10a, bernstrasse 7 8
    match Regex::new(r"(.*?\s\d*?\s?\w?)[/,\-\s]")
        .unwrap()
        .find(&street)
    {
        // but not bernstrasse 7 A
        Some(mat) if !Regex::new(r"\s\d+[/,\-\s]\w$").unwrap().is_match(&street) => mat.as_str(),
        _ => street.as_str(),
    }
    .trim()
    .to_string()
}

fn does_contain_numbers(street: &str) -> bool {
    street.chars().map(char::is_numeric).count() > 0
}

pub struct StreetMatcher {
    street: String,
    sensitivity: f64,
    file_sensitivity: f64,
}

impl StreetMatcher {
    pub fn new(street: &str, sens: Option<f64>, file_sens: Option<f64>) -> Self {
        if !does_contain_numbers(street) {
            panic!(
                "Argument 'street' must contain street number! Got: '{}'",
                street
            );
        }
        Self {
            street: clean_street(street),
            sensitivity: sens.unwrap_or(SENSITIVITY),
            file_sensitivity: file_sens.unwrap_or(FILE_SENSITIVITY),
        }
    }

    fn _find_matches_in_dir(&self, dir: &Path, is_first_letters_eq: bool) -> Vec<Candidate> {
        TextMatcher::find_matches_in_dir(
            self.sensitivity,
            KEEP,
            &self.street,
            dir.to_path_buf(),
            None,
            is_first_letters_eq,
        )
    }

    fn _search_in_dir(&self, dir: &Path, file_cand: Option<PathBuf>) -> MatchedStreet {
        let mut mat = self._find_matches_in_dir(dir, true);
        if mat.is_empty() {
            mat = self._find_matches_in_dir(dir, false);
        }
        if !mat.is_empty() {
            MatchedStreet {
                street: Some(mat[0].text.clone()),
                file_found: file_cand.and_then(|file_found| {
                    let potential_cand = Candidate {
                        file_found,
                        ..mat[0].clone()
                    };
                    if mat.iter().filter(|cand| potential_cand == **cand).count() > 0 {
                        Some(potential_cand.file_found)
                    } else {
                        None
                    }
                }),
            }
        } else {
            MatchedStreet {
                street: None,
                file_found: None,
            }
        }
    }

    fn _find_matches(&self, dir: &Path, file: Option<PathBuf>) -> MatchedStreet {
        file.map_or_else(
            || self._search_in_dir(dir, None),
            |file| match TextMatcher::new(self.file_sensitivity, KEEP, false)
                .find_matches_in_file(&self.street, &file)
            {
                Ok(mat) if !mat.is_empty() => MatchedStreet {
                    street: Some(mat[0].text.clone()),
                    file_found: Some(file),
                },
                _ => self._search_in_dir(dir, Some(file)),
            },
        )
    }

    pub fn match_by_plz(&self, plz: Option<usize>) -> MatchedStreet {
        self._find_matches(
            &PathBuf::from(PATH_TO_PLZS_DIR),
            plz.map(|plz| PathBuf::from(format!("{}{}", PATH_TO_PLZS_DIR, plz))),
        )
    }

    pub fn match_by_place(&self, place: Option<&str>) -> MatchedStreet {
        self._find_matches(
            &PathBuf::from(PATH_TO_PLACES_DIR),
            place.and_then(|place| {
                StreetMatcher::_match_place(place).map(|candidate| {
                    PathBuf::from(format!("{}{}", PATH_TO_PLACES_DIR, candidate.text))
                })
            }),
        )
    }

    fn _match_place(place: &str) -> Option<Candidate> {
        match TextMatcher::new(PLACE_SEARCH_SENSITIVITY, KEEP, false).find_matches_in_file(
            &String::from(place).to_lowercase(),
            &PathBuf::from_str(PATH_TO_PLACES).expect("places.txt file exists"),
        ) {
            Ok(candidates) if !candidates.is_empty() => Some(candidates[0].clone()),
            _ => None,
        }
    }
}
