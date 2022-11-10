//! This module provides matching on official Switzerland streets

// TODO: if more than 1 candidate with the same similarity for places
// TODO: add PLZ and Place structs and define preprocessing/to dir/file inside structs (maybe generics?)
#![allow(dead_code)]
use crate::{
    candidate::Candidate,
    text_matcher::{MatchAlgo, TextMatcher, PUNCTUATIONS},
};

use regex::Regex;
use std::path::{Path, PathBuf};

const SENSITIVITY: f64 = 0.6;
const FILE_SENSITIVITY: f64 = 0.87;
const PLACE_SEARCH_SENSITIVITY: f64 = 0.7;
const NUM_TO_KEEP: usize = 50;
const PATH_TO_PLACES: &str = "./test_data/places.txt";
pub(crate) const PATH_TO_PLZS_DIR: &str = "./test_data/plzs/";
pub(crate) const PATH_TO_PLACES_DIR: &str = "./test_data/places/";

pub struct SwissStreet {
    pub value: String,
}

impl SwissStreet {
    pub fn new(street: &str) -> Self {
        if !SwissStreet::contains_numbers(street) {
            panic!(
                "Argument 'street' must contain street number! Got: '{}'",
                street
            );
        }
        Self {
            value: SwissStreet::clean(street),
        }
    }

    #[inline]
    pub(crate) fn contains_numbers(street: &str) -> bool {
        street.chars().filter(|ch| ch.is_numeric()).count() > 0
    }

    #[inline]
    pub(crate) fn clean(street: &str) -> String {
        let mut street = street.trim().to_lowercase();
        // Matches: '76 chemin des clos' or 'a4 résidence du golf'
        if SwissStreet::starts_with_street_number(&street) {
            let mut parts = street.split_whitespace();
            let number = parts
                .next()
                .expect("the string format is correct due to regex");
            street = format!("{} {}", parts.collect::<Vec<&str>>().join(" "), number);
        }
        // Matches: eisfeldstrasse 21/23, milchstrasse 2-10a, milchstrasse 2,10a, bernstrasse 7 8
        match Regex::new(r"(.*?\s\d*?\s?[a-zA-Z]?)[\./,\-\+\s–+]")
            .unwrap()
            .find(&street)
        {
            // but not bernstrasse 7 A
            Some(mat) if !Regex::new(r"\s\d*?\s[a-zA-Z]$").unwrap().is_match(&street) => {
                mat.as_str()
            }
            _ => street.as_str(),
        }
        .trim()
        .replace(PUNCTUATIONS, "")
    }

    #[inline]
    pub(crate) fn starts_with_street_number(street: &str) -> bool {
        Regex::new(r"^\d+,?\s.+").unwrap().is_match(street)
            || Regex::new(r"^\w\d+,?\s").unwrap().is_match(street)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MatchedStreet {
    pub street: Option<String>,
    pub file_found: Option<PathBuf>,
}

impl MatchedStreet {
    pub fn new(street: String, file_found: PathBuf) -> Self {
        Self {
            street: Some(street),
            file_found: Some(file_found),
        }
    }

    #[inline]
    pub(crate) fn empty() -> Self {
        Self {
            street: None,
            file_found: None,
        }
    }

    #[inline]
    fn from_candidates(candidates: &[Candidate], file_candidate: Option<PathBuf>) -> Self {
        if candidates.is_empty() {
            return Self::empty();
        }
        Self {
            street: Some(candidates[0].text.clone()),
            file_found: file_candidate.and_then(|file_found| {
                let potential_candidate = Candidate {
                    file_found,
                    ..candidates[0].clone()
                };
                if candidates.contains(&potential_candidate) {
                    Some(potential_candidate.file_found)
                } else {
                    None
                }
            }),
        }
    }
}

pub struct StreetMatcher {
    pub sensitivity: f64,
    pub file_sensitivity: f64,
}

impl Default for StreetMatcher {
    fn default() -> Self {
        Self {
            sensitivity: SENSITIVITY,
            file_sensitivity: FILE_SENSITIVITY,
        }
    }
}

impl StreetMatcher {
    /// StreetMatcher constructor gives possibility to finetune the matching process
    /// by setting custom sensitivity and file_sensitivity values (each from 0.0 - keep all matches, to 1.0 - keep only direct matches).
    /// However, it's recommended to keep default values, i.e. sensitivity == 0.6 - dir seacrh, and file_sensitivity == 0.87 - file search
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_matcher_rs::StreetMatcher;
    /// #
    /// # fn main() {
    /// #     let sm = StreetMatcher::new(None, None);
    /// #     assert_eq!(sm.match_by_plz("qu du seujet 36", Some(1201)).street.unwrap(), "quai du seujet 36".to_string());
    /// #     assert_eq!(sm.match_by_place("aarstr. 76", Some("Bern")).street.unwrap(), "aarstrasse 76".to_string());
    /// # }
    /// ```
    pub fn new(sensitivity: Option<f64>, file_sensitivity: Option<f64>) -> Self {
        Self {
            sensitivity: sensitivity.unwrap_or(SENSITIVITY),
            file_sensitivity: file_sensitivity.unwrap_or(FILE_SENSITIVITY),
        }
    }

    fn _find_matches_in_dir(
        &self,
        street: &SwissStreet,
        dir: &Path,
        is_first_letters_eq: bool,
    ) -> Vec<Candidate> {
        TextMatcher::find_matches_in_dir(
            self.sensitivity,
            NUM_TO_KEEP,
            &street.value,
            dir,
            None,
            Some(is_first_letters_eq),
            MatchAlgo::default(),
        )
    }

    fn _search_in_dir(
        &self,
        street: &SwissStreet,
        dir: &Path,
        file_candidate: Option<PathBuf>,
    ) -> MatchedStreet {
        let candidates = {
            let candidates = self._find_matches_in_dir(street, dir, true);
            if !candidates.is_empty() {
                candidates
            } else {
                self._find_matches_in_dir(street, dir, false)
            }
        };
        MatchedStreet::from_candidates(&candidates, file_candidate)
    }

    fn _find_matches(
        &self,
        street: &SwissStreet,
        dir: &Path,
        file: Option<PathBuf>,
    ) -> MatchedStreet {
        file.map_or_else(
            || self._search_in_dir(street, dir, None),
            |file| match TextMatcher::new(self.file_sensitivity, NUM_TO_KEEP, MatchAlgo::default())
                .find_matches_in_file(&street.value, &file, None)
            {
                Ok(mat) if !mat.is_empty() => MatchedStreet::new(mat[0].text.clone(), file),
                _ => self._search_in_dir(street, dir, Some(file)),
            },
        )
    }

    /// Search for a candidate street(s) to a target street within a Postal Code (`plz`).
    /// All official street candidates here grouped into files named by a Postal Code.
    /// `plz` must be a valid Switzerland Postal Code represented officially by government.
    /// Otherwise, if `plz` did not match any of existings Postal Codes in the directory,
    /// the search on the WHOLE directory (all files inside a directory) is provided.
    /// Also, if a candidate was not found within a given `plz`, the same logic (search on all files) is applied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_matcher_rs::StreetMatcher;
    /// #
    /// # fn main() {
    /// #     let mat = StreetMatcher::new(None, None).match_by_plz("qu du seujet 36", Some(1201));
    /// #     assert_eq!(mat.street.unwrap(), "quai du seujet 36".to_string());
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `street` does not contain a number (as each valid street MUST contain an any number)
    pub fn match_by_plz(&self, street: &str, plz: Option<usize>) -> MatchedStreet {
        self._find_matches(
            &SwissStreet::new(street),
            &PathBuf::from(PATH_TO_PLZS_DIR),
            plz.map(|plz| PathBuf::from(format!("{}{}", PATH_TO_PLZS_DIR, plz))),
        )
    }

    /// Search for a candidate street(s) to a target street within a Swiss peace of territory, assigned to the Postal Code (called `place`).
    /// All official street candidates here grouped into files named by `place`.
    /// `place` could be an invalid name. In this case, the matcher will try to search for `place` candidate inside a `places.txt` file.
    /// If `place` did not match any of existings Postal Codes in the directory,
    /// the search on the WHOLE directory (all files inside a directory) is provided.
    /// Also, if a candidate was not found within a given `plz`, the same logic (search on all files) is applied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_matcher_rs::StreetMatcher;
    /// #
    /// # fn main() {
    /// #     let mat = StreetMatcher::new(None, None).match_by_place("aarstr. 76", Some("Bern"));
    /// #     assert_eq!(mat.street.unwrap(), "aarstrasse 76".to_string());
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `street` does not contain a number (as each valid street MUST contain an any number)
    pub fn match_by_place(&self, street: &str, place: Option<&str>) -> MatchedStreet {
        self._find_matches(
            &SwissStreet::new(street),
            &PathBuf::from(PATH_TO_PLACES_DIR),
            place
                .and_then(StreetMatcher::_match_place)
                .map(|place| PathBuf::from(format!("{}{}", PATH_TO_PLACES_DIR, place))),
        )
    }

    fn _match_place(place: &str) -> Option<String> {
        // TODO: consider removing PUNCTUATIONS and '/', and compare places like that
        //       the problem is that we cannot revert place name back (as some places come with '(', ')', etc.
        //       Possibly, we need to remove PUNCTUATIONS while comparing target string with the candidate string in-place
        match TextMatcher::new(
            PLACE_SEARCH_SENSITIVITY,
            NUM_TO_KEEP,
            MatchAlgo::JaroWinkler,
        )
        .find_matches_in_file(
            &place.trim().to_lowercase(),
            &PathBuf::from(PATH_TO_PLACES),
            None,
        ) {
            Ok(candidates) if !candidates.is_empty() => {
                Some(candidates[0].text.replace(' ', "_").replace('/', "%2C"))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STREET_WITHOUT_NUMBERS: &str = "Bernstrasse";
    const STREET_WITH_NUMBER: &str = "Bernstrasse 7";

    #[test]
    #[ignore]
    fn max_sensitivity() {
        // Some random string in the input
        let street = "FdsfdsfsdfssFSDfdsfsdfsBernstrasse 7";
        assert_eq!(
            StreetMatcher::new(Some(1.0), Some(1.0)).match_by_plz(street, None),
            MatchedStreet::empty()
        );
    }

    #[test]
    fn street_contains_numbers() {
        assert!(SwissStreet::contains_numbers(STREET_WITH_NUMBER))
    }

    #[test]
    fn street_does_not_contain_numbers() {
        assert!(!SwissStreet::contains_numbers(STREET_WITHOUT_NUMBERS))
    }

    #[test]
    #[should_panic(expected = "must contain street number")]
    fn no_numbers_in_street_plz() {
        StreetMatcher::default().match_by_plz(STREET_WITHOUT_NUMBERS, None);
    }

    #[test]
    #[should_panic(expected = "must contain street number")]
    fn no_numbers_in_street_place() {
        StreetMatcher::default().match_by_place(STREET_WITHOUT_NUMBERS, None);
    }

    fn assert_clean_street(expected_street: &str, street_to_clean: &str) {
        assert_eq!(
            expected_street.to_string(),
            SwissStreet::clean(street_to_clean)
        );
    }

    #[test]
    fn clean_street() {
        assert_clean_street("bernstrasse 7", "   Bernstrasse 7   ");
        assert_clean_street("bernstrasse a4", "   a4 Bernstrasse   ");
        assert_clean_street("bernstrasse 4", "   4 Bernstrasse   ");
        assert_clean_street("bernstrasse 4a", "   Bernstrasse 4a, 5, 6   ");
        assert_clean_street("bernstrasse 4a", "   Bernstrasse 4a 5 6   ");
        assert_clean_street("bernstrasse 4a", "   Bernstrasse 4a-5-6   ");
        assert_clean_street("bernstrasse 4a", "   Bernstrasse 4a/5/6   ");
        assert_clean_street("bernstrasse 4a", "   Bernstrasse 4a. 5 6   ");
        assert_clean_street("bernstrasse 4 a", "  Bernstrasse 4 A fasdfs");
    }

    #[test]
    fn match_with_place() {
        assert_eq!(
            StreetMatcher::default().match_by_place("ch de saint-cierges 3", Some("bercher")),
            MatchedStreet::new(
                "chemin de saint-cierges 3".to_string(),
                PathBuf::from("./test_data/places/bercher")
            )
        );
    }

    #[test]
    #[ignore]
    fn match_without_place() {
        let mat = StreetMatcher::default().match_by_place("ch de saint-cierges 3", None);
        assert_eq!(mat.street.unwrap(), "chemin de saint-cierges 3".to_string());
    }

    #[test]
    fn match_with_plz() {
        let mat = StreetMatcher::default().match_by_plz("qu du seujet 36", Some(1201));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(
            mat.file_found.unwrap(),
            PathBuf::from("./test_data/plzs/1201")
        );
    }

    #[test]
    #[ignore]
    fn match_without_plz() {
        let mat = StreetMatcher::default().match_by_plz("qu du seujet 36", None);
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    #[ignore]
    fn match_with_wrong_plz() {
        let mat = StreetMatcher::default().match_by_plz("qu du seujet 36", Some(1231231));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    #[ignore]
    fn match_with_wrong_first_word() {
        let mat = StreetMatcher::default().match_by_plz("u du seujet 36", Some(1201));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(
            mat.file_found.unwrap(),
            PathBuf::from("./test_data/plzs/1201")
        )
    }

    #[test]
    #[ignore]
    fn match_with_wrong_first_word_no_plz() {
        let mat = StreetMatcher::default().match_by_plz("u du seujet 36", None);
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    #[ignore]
    fn match_with_wrong_first_word_wrong_plz() {
        let mat = StreetMatcher::default().match_by_plz("u du seujet 36", Some(2132131));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }
}
