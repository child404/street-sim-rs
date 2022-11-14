//! This module provides matching on official Switzerland streets

// TODO: if more than 1 candidate with the same similarity for places
// #![allow(dead_code)]
use crate::{
    candidate::{Candidate, Sens, SimResult, Text},
    text_sim::{self, Config, SimAlgo},
};

use regex::Regex;
use std::{fs, path::PathBuf};
use toml::Value;

const PLACE_SENS: f64 = 0.6;
const PATH_TO_PLACES: &str = "./test_data/places.txt";
const PATH_TO_STREET_NAMES: &str = "./test_data/streets_data/street_names.txt";
const PATH_TO_STREETS_DATA: &str = "./test_data/streets_data";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Place(pub(crate) String);

impl Place {
    pub fn new(value: &str) -> Self {
        let cfg = Config::new(Sens::new(PLACE_SENS), 1, SimAlgo::JaroWinkler, None);
        Self(
            text_sim::fast_cmp_with_file(&Text::new(value), &PathBuf::from(PATH_TO_PLACES), &cfg)
                .map_or(value.to_owned(), |candidates| candidates[0].text.to_owned()),
        )
    }
}

impl ToString for Place {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Plz(pub(crate) String);

impl ToString for Plz {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

impl Plz {
    pub fn new(value: usize) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Street {
    pub value: Text,
    pub street_name: String,
}

impl Street {
    pub fn new(street: &str, sens: Option<Sens>) -> Option<Self> {
        if !Self::contains_numbers(street) {
            panic!(
                "Argument 'street' must contain street number! Got: '{}'",
                street
            );
        }
        let street = Text::new(&Self::clean(street));
        find_street_name(&street, sens.unwrap_or_default())
            .ok()
            .map(|candidates| Self {
                value: street,
                street_name: candidates[0].text.to_owned(),
            })
    }

    #[inline]
    pub(crate) fn contains_numbers(street: &str) -> bool {
        street.chars().filter(|ch| ch.is_numeric()).count() > 0
    }

    #[inline]
    pub(crate) fn clean(street: &str) -> String {
        let mut street = street.trim().to_owned();
        // Matches: '76 chemin des clos' or 'a4 résidence du golf'
        if Self::starts_with_number(&street) {
            let (num, street_name) = street.split_once(' ').expect("matched by regexp");
            street = format!("{} {}", street_name, num);
        }
        // Matches: eisfeldstrasse 21/23, milchstrasse 2-10a, milchstrasse 2,10a, bernstrasse 7 8
        match Regex::new(r"(.*?\s\d*?\s?[a-zA-Z]?)[\./,\-\+\s–\\]")
            .unwrap()
            .find(&street)
        {
            // but not bernstrasse 7 A
            Some(mat) if !Regex::new(r"\s\d*?\s[a-zA-Z]$").unwrap().is_match(&street) => {
                mat.as_str().to_string()
            }
            _ => street,
        }
    }

    #[inline]
    pub(crate) fn starts_with_number(street: &str) -> bool {
        Regex::new(r"^\d+,?\s.+").unwrap().is_match(street)
            || Regex::new(r"^\w\d+,?\s").unwrap().is_match(street)
    }
}

#[inline]
fn find_street_name(street: &Text, sens: Sens) -> SimResult {
    let cfg = Config::new(sens, 1, SimAlgo::Levenshtein, None);
    text_sim::cmp_with_arr(&filter_distant_streets(street, sens), street, &cfg)
}

#[inline]
fn filter_distant_streets(street: &Text, sens: Sens) -> Vec<String> {
    let cfg = Config::new(sens, 500, SimAlgo::Jaro, None);
    text_sim::fast_cmp_with_file(street, &PathBuf::from(PATH_TO_STREET_NAMES), &cfg)
        .unwrap_or_default()
        .iter()
        .map(|c| c.text.to_owned())
        .collect()
}

type ValuesByLocation<T> = (Vec<String>, Option<T>);
type CandidateByLocation<T> = (Option<Candidate>, Option<T>);

struct StreetFile {
    values: Value,
}

impl StreetFile {
    pub fn new(street: &Street) -> Self {
        let filename = format!(
            "{}/{}.toml",
            PATH_TO_STREETS_DATA,
            street.street_name.replace('/', "%2C")
        );
        Self {
            values: toml::from_str::<Value>(
                &fs::read_to_string(filename).expect("street file exists"),
            )
            .expect("correct toml signature"),
        }
    }

    fn get_all_values(&self) -> Vec<String> {
        let mut values = self
            .values
            .as_table()
            .expect("correct table structure")
            .iter()
            .flat_map(|(_, v)| v.as_array().unwrap())
            .map(|x| x.to_string().replace('\"', ""))
            .collect::<Vec<String>>();
        values.sort();
        values.dedup();
        values
    }

    fn try_get_values_by<T>(&self, location: T) -> ValuesByLocation<T>
    where
        T: ToString,
    {
        self.values.get(&location.to_string()).map_or_else(
            || (self.get_all_values(), None),
            |values| {
                (
                    values
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.to_string().replace('\"', ""))
                        .collect::<Vec<String>>(),
                    Some(location),
                )
            },
        )
    }
}

#[inline]
fn cmp_with_arr<T>(
    arr: &[String],
    street: &Street,
    cfg: &Config,
    location: Option<T>,
) -> CandidateByLocation<T>
where
    T: ToString,
{
    match text_sim::cmp_with_arr(arr, &street.value, cfg) {
        Ok(mat) => (Some(mat[0].to_owned()), location),
        Err(_) => (None, None),
    }
}

/// Search for a candidate street(s) to a target street within a Postal Code (`plz`).
/// All official street candidates here grouped into files named by a Postal Code.
/// `plz` must be a valid Switzerland Postal Code represented officially by government.
/// Otherwise, if `plz` did not match any of existings Postal Codes in the directory,
/// the search on the WHOLE directory (all files inside a directory) is provided.
/// Also, if a candidate was not found within a given `plz`, the same logic (search on all files) is applied.
///
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
/// # use text_matcher_rs::{Plz, Street, street_sim};
/// #
/// # fn main() {
/// #     let street = Street::new("qu du seujet 36", None).unwrap();
/// #     let mat = street_sim::find_matches(&street, Some(Plz::new(1201)), None);
/// #     assert_eq!(mat.0.unwrap().text, "quai du seujet 36".to_string());
/// # }
/// ```
///
/// ```rust
/// # use text_matcher_rs::{Place, Street, street_sim};
/// #
/// # fn main() {
/// #     let street = Street::new("aarstrasse 76", None).unwrap();
/// #     let mat = street_sim::find_matches(&street, Some(Place::new("Bern")), None);
/// #     assert_eq!(mat.0.unwrap().text, "aarstrasse 76".to_string());
/// # }
/// ```
///
/// # Panics
///
/// Panics if `street` does not contain a number (as each valid street MUST contain an any number)
pub fn find_matches<T>(
    street: &Street,
    location: Option<T>,
    sens: Option<Sens>,
) -> CandidateByLocation<T>
where
    T: ToString,
{
    let cfg = Config::new(sens.unwrap_or_default(), 1, SimAlgo::default(), None);
    let street_file = StreetFile::new(street);
    location.map_or_else(
        || cmp_with_arr(&street_file.get_all_values(), street, &cfg, None),
        |location| {
            let (streets_to_match, location) = street_file.try_get_values_by(location);
            cmp_with_arr(&streets_to_match, street, &cfg, location)
        },
    )
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
        assert_eq!(Street::new(street, None), None);
    }

    #[test]
    fn street_contains_numbers() {
        assert!(Street::contains_numbers(STREET_WITH_NUMBER))
    }

    #[test]
    fn street_does_not_contain_numbers() {
        assert!(!Street::contains_numbers(STREET_WITHOUT_NUMBERS))
    }

    #[test]
    #[should_panic(expected = "must contain street number")]
    fn no_numbers_in_street_plz() {
        find_matches::<Plz>(
            &Street::new(STREET_WITHOUT_NUMBERS, None).unwrap(),
            None,
            None,
        );
    }

    fn assert_clean_street(expected_street: &str, street_to_clean: &str) {
        assert_eq!(
            expected_street.to_string(),
            Street::new(street_to_clean, None).unwrap().value.cleaned
        );
    }

    #[test]
    fn clean_street() {
        assert_clean_street("bernstrasse7", "   Bernstrasse 7   ");
        assert_clean_street("bernstrassea4", "   a4 Bernstrasse   ");
        assert_clean_street("bernstrasse4", "   4 Bernstrasse   ");
        assert_clean_street("bernstrasse4a", "   Bernstrasse 4a, 5, 6   ");
        assert_clean_street("bernstrasse4a", "   Bernstrasse 4a 5 6   ");
        assert_clean_street("bernstrasse4a", "   Bernstrasse 4a-5-6   ");
        assert_clean_street("bernstrasse4a", "   Bernstrasse 4a/5/6   ");
        assert_clean_street("bernstrasse4a", "   Bernstrasse 4a. 5 6   ");
        assert_clean_street("bernstrasse4a", "  Bernstrasse 4 A fasdfs");
    }

    #[test]
    fn match_with_place() {
        let location = Place::new("bercher");
        assert_eq!(
            find_matches(
                &Street::new("ch de saint-cierges 3", None).unwrap(),
                Some(location.to_owned()),
                None
            ),
            (
                Some(Candidate::from("chemin de saint-cierges 3")),
                Some(location)
            )
        );
    }

    #[test]
    #[ignore]
    fn match_without_place() {
        let mat = find_matches::<Place>(
            &Street::new("ch de saint-cierges 3", None).unwrap(),
            None,
            None,
        );
        assert_eq!(
            mat,
            (Some(Candidate::from("chemin de saint-cierges 3")), None)
        );
    }

    #[test]
    fn match_with_plz() {
        let location = Plz::new(1201);
        let mat = find_matches(
            &Street::new("qu du seujet 36", None).unwrap(),
            Some(location.to_owned()),
            None,
        );
        assert_eq!(
            mat,
            (Some(Candidate::from("quai du seujet 36")), Some(location))
        );
    }

    #[test]
    #[ignore]
    fn match_without_plz() {
        let mat = find_matches::<Plz>(&Street::new("qu du seujet 36", None).unwrap(), None, None);
        assert_eq!(mat, (Some(Candidate::from("quai du seujet 36")), None));
    }

    #[test]
    #[ignore]
    fn match_with_wrong_plz() {
        let mat = find_matches(
            &Street::new("qu du seujet 36", None).unwrap(),
            Some(Plz::new(1231231)),
            None,
        );
        assert_eq!(mat, (Some(Candidate::from("quai du seujet 36")), None));
    }

    #[test]
    #[ignore]
    fn match_with_wrong_first_word() {
        let location = Plz::new(1201);
        let mat = find_matches(
            &Street::new("uai du seujet 36", None).unwrap(),
            Some(location.to_owned()),
            None,
        );
        assert_eq!(
            mat,
            (Some(Candidate::from("quai du seujet 36")), Some(location))
        );
    }

    #[test]
    #[ignore]
    fn match_with_wrong_first_word_no_plz() {
        let mat = find_matches::<Plz>(&Street::new("uai du seujet 36", None).unwrap(), None, None);
        assert_eq!(mat, (Some(Candidate::from("quai du seujet 36")), None));
    }

    #[test]
    #[ignore]
    fn match_with_wrong_first_word_wrong_plz() {
        let location = Plz::new(2132131);
        let mat = find_matches(
            &Street::new("uai du seujet 36", None).unwrap(),
            Some(location),
            None,
        );
        assert_eq!(mat, (Some(Candidate::from("quai du seujet 36")), None));
    }
}