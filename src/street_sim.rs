//! This module provides matching on official Switzerland streets

use crate::{
    candidate::{Candidate, Error as CandidateError, Sens, SimResult, Text},
    text_sim::{self, Config, SimAlgo},
};

use regex::Regex;
use std::{fs, io, path::PathBuf, process};
use toml::Value;

const PLACE_SENS: f64 = 0.6;
const NUM_TO_KEEP_FILTERED: usize = 500;
const PATH_TO_PLACES: &str = "./test_data/places.txt";
const PATH_TO_STREET_NAMES: &str = "./test_data/streets_data/street_names.txt";
const PATH_TO_STREETS_DATA: &str = "./test_data/streets_data";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Place(pub(crate) String);

impl Place {
    pub fn new(value: &str) -> Self {
        let cfg = Config::new(Sens::new(PLACE_SENS), 1, SimAlgo::JaroWinkler, None);
        Self(
            text_sim::fast_cmp_with_file(
                &Text::new(value.to_string()),
                &PathBuf::from(PATH_TO_PLACES),
                &cfg,
            )
            .map_or(String::new(), |candidates| candidates[0].text.clone()),
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

#[inline]
fn filter_distant_streets(street: &Text, cfg: &Config) -> Vec<String> {
    text_sim::fast_cmp_with_file(street, &PathBuf::from(PATH_TO_STREET_NAMES), cfg)
        .unwrap_or_default()
        .into_iter()
        .map(|c| c.text)
        .collect()
}

#[inline]
fn find_street_name(street: &Text, cfg: &Config) -> SimResult {
    let cfg_for_filter = Config {
        num_to_keep: NUM_TO_KEEP_FILTERED,
        sim_func: SimAlgo::Jaro.into(),
        ..*cfg
    };
    text_sim::cmp_with_arr(
        &filter_distant_streets(street, &cfg_for_filter),
        street,
        cfg,
    )
}

#[derive(Debug)]
pub struct Street {
    pub value: Text,
    pub file: StreetFile,
}

impl PartialEq for Street {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl Street {
    pub fn new(street: &str, cfg: Option<Config>) -> Result<Self, CandidateError> {
        if !Self::contains_numbers(street) {
            eprintln!(
                "Argument 'street' must contain street number! Got: '{}'",
                street
            );
            process::exit(1);
        }
        let street = Text::new(Self::clean(street));
        Ok(Self {
            file: StreetFile::new(&find_street_name(&street, &cfg.unwrap_or_default())?[0].text)?,
            value: street,
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

#[derive(Debug)]
pub struct StreetFile {
    values: Value,
}

impl StreetFile {
    pub fn new(street_name: &str) -> io::Result<Self> {
        let filename = format!(
            "{}/{}.toml",
            PATH_TO_STREETS_DATA,
            street_name.replace('/', "%2C")
        );
        Ok(Self {
            values: toml::from_str::<Value>(&fs::read_to_string(filename)?)?,
        })
    }

    #[inline]
    fn iter_over_strings_from(value: &Value) -> impl Iterator<Item = String> + '_ {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.to_string().replace('\"', ""))
    }

    #[inline]
    fn get_all_values(&self) -> Vec<String> {
        let mut values = self
            .values
            .as_table()
            .expect("correct table structure")
            .iter()
            .flat_map(|(_, v)| Self::iter_over_strings_from(v))
            .collect::<Vec<String>>();
        values.sort();
        values.dedup();
        values
    }

    fn get_values_by<T>(&self, location: Option<&T>) -> (Vec<String>, bool)
    where
        T: ToString,
    {
        if let Some(location) = location {
            if let Some(v) = self.values.get(&location.to_string()) {
                return (Self::iter_over_strings_from(v).collect(), true);
            }
        }
        (self.get_all_values(), false)
    }
}

pub struct StreetConfig<T> {
    location: Option<T>,
    cfg: Config,
}

impl<T> Default for StreetConfig<T> {
    fn default() -> Self {
        Self {
            location: None,
            cfg: Config::default(),
        }
    }
}

impl<T> StreetConfig<T> {
    pub fn new(location: Option<T>, sens: f64, num_to_keep: usize, algo: SimAlgo) -> Self {
        Self {
            location,
            cfg: Config::new(Sens::new(sens), num_to_keep, algo, None),
        }
    }

    pub fn from(location: T) -> Self {
        Self {
            location: Some(location),
            cfg: Config::default(),
        }
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
/// # use street_sim_rs::{Plz, Street, street_sim};
/// #
/// # fn main() {
/// #     let street = Street::new("qu du seujet 36", None).unwrap();
/// #     let mat = street_sim::find_matches(&street, Some(Plz::new(1201)), None);
/// #     assert_eq!(mat.0.unwrap().text, "quai du seujet 36".to_string());
/// # }
/// ```
///
/// ```rust
/// # use street_sim_rs::{Place, Street, street_sim};
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
    street_cfg: StreetConfig<T>,
) -> io::Result<(Option<Candidate>, Option<T>)>
where
    T: ToString,
{
    let (street_candidates, is_found_in_loc) =
        street.file.get_values_by(street_cfg.location.as_ref());
    Ok((
        text_sim::cmp_with_arr(&street_candidates, &street.value, &street_cfg.cfg)
            .ok()
            .map(|mat| mat[0].clone()),
        if is_found_in_loc {
            street_cfg.location
        } else {
            None
        },
    ))
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
        let street = Street::new(street, None);
        assert!(
            matches!(street, Err(_)),
            "Expected NotFound error, but the value was {:?}",
            street
        );
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
            StreetConfig::default(),
        )
        .unwrap();
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
                StreetConfig::from(location.to_owned()),
            )
            .unwrap(),
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
            StreetConfig::default(),
        );
        assert_eq!(
            mat.unwrap(),
            (Some(Candidate::from("chemin de saint-cierges 3")), None)
        );
    }

    #[test]
    fn match_with_plz() {
        let location = Plz::new(1201);
        let mat = find_matches(
            &Street::new("qu du seujet 36", None).unwrap(),
            StreetConfig::from(location.to_owned()),
        );
        assert_eq!(
            mat.unwrap(),
            (Some(Candidate::from("quai du seujet 36")), Some(location))
        );
    }

    #[test]
    #[ignore]
    fn match_without_plz() {
        let mat = find_matches::<Plz>(
            &Street::new("qu du seujet 36", None).unwrap(),
            StreetConfig::default(),
        );
        assert_eq!(
            mat.unwrap(),
            (Some(Candidate::from("quai du seujet 36")), None)
        );
    }

    #[test]
    #[ignore]
    fn match_with_wrong_plz() {
        let mat = find_matches(
            &Street::new("qu du seujet 36", None).unwrap(),
            StreetConfig::from(Plz::new(1231231)),
        );
        assert_eq!(
            mat.unwrap(),
            (Some(Candidate::from("quai du seujet 36")), None)
        );
    }

    #[test]
    #[ignore]
    fn match_with_wrong_first_word() {
        let location = Plz::new(1201);
        let mat = find_matches(
            &Street::new("uai du seujet 36", None).unwrap(),
            StreetConfig::from(location.to_owned()),
        );
        assert_eq!(
            mat.unwrap(),
            (Some(Candidate::from("quai du seujet 36")), Some(location))
        );
    }

    #[test]
    #[ignore]
    fn match_with_wrong_first_word_no_plz() {
        let mat = find_matches::<Plz>(
            &Street::new("uai du seujet 36", None).unwrap(),
            StreetConfig::default(),
        );
        assert_eq!(
            mat.unwrap(),
            (Some(Candidate::from("quai du seujet 36")), None)
        );
    }

    #[test]
    #[ignore]
    fn match_with_wrong_first_word_wrong_plz() {
        let location = Plz::new(2132131);
        let mat = find_matches(
            &Street::new("uai du seujet 36", None).unwrap(),
            StreetConfig::from(location),
        );
        assert_eq!(
            mat.unwrap(),
            (Some(Candidate::from("quai du seujet 36")), None)
        );
    }
}
