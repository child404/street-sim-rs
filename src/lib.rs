//! This crate is aimed to be a simple and fast solution for text-matching from the file with
//! more than 2 millions of lines, especially for streets in Switzerland.
//!
//! Also, it serves as my first Rust project used for work and published out to the people
mod candidate;
mod street_matcher;
mod text_matcher;

pub use candidate::Candidate;
pub use street_matcher::StreetMatcher;
pub use text_matcher::TextMatcher;

#[cfg(test)]
mod tests {
    use crate::{street_matcher, StreetMatcher, TextMatcher};
    use pretty_assertions::assert_eq;
    use std::{path::PathBuf, str::FromStr};

    const DATA_FILE: &str = "./test_data/plzs/1201";
    const DATA_DIR: &str = "./test_data/plzs/";

    #[test]
    fn street_contains_numbers() {
        let street = "Bernstrasse 7";
        assert!(
            street_matcher::contains_numbers(street),
            "Street does not contain numbers, value was {}",
            street
        )
    }

    #[test]
    fn street_does_not_contain_numbers() {
        let street = "Bernstrasse";
        assert!(
            !street_matcher::contains_numbers(street),
            "Street contain numbers, value was {}",
            street
        )
    }

    #[test]
    #[should_panic(expected = "must contain street number")]
    fn no_numbers_in_street_by_plz() {
        let street = "Bernstrasse";
        StreetMatcher::new(None, None).match_by_plz(street, None);
    }

    #[test]
    #[should_panic(expected = "must contain street number")]
    fn no_numbers_in_street_by_place() {
        let street = "Bernstrasse";
        StreetMatcher::new(None, None).match_by_place(street, None);
    }

    #[test]
    fn clean_street() {
        let street = "   Bernstrasse 7   ";
        assert_eq!(
            street_matcher::clean_street(street),
            "bernstrasse 7".to_string()
        );

        let street = "   a4 Bernstrasse   ";
        assert_eq!(
            street_matcher::clean_street(street),
            "bernstrasse a4".to_string()
        );

        let street = "   4 Bernstrasse   ";
        assert_eq!(
            street_matcher::clean_street(street),
            "bernstrasse 4".to_string()
        );

        let street = "   Bernstrasse 4a, 5, 6   ";
        assert_eq!(
            street_matcher::clean_street(street),
            "bernstrasse 4a".to_string()
        );

        // FIXME: doesn't work for \s separator
        let street = "   Bernstrasse 4a 5 6   ";
        assert_eq!(
            street_matcher::clean_street(street),
            "bernstrasse 4a".to_string()
        );

        let street = "   Bernstrasse 4a-5-6   ";
        assert_eq!(
            street_matcher::clean_street(street),
            "bernstrasse 4a".to_string()
        );

        let street = "   Bernstrasse 4a/5/6   ";
        assert_eq!(
            street_matcher::clean_street(street),
            "bernstrasse 4a".to_string()
        );

        // TODO: add tests for Bernstrasse 7 A
        let street = "   Bernstrasse 4a. 5 6   ";
        assert_eq!(
            street_matcher::clean_street(street),
            "bernstrasse 4a".to_string()
        );
    }

    #[test]
    fn street_matcher_with_place() {
        let mat = StreetMatcher::default().match_by_place("ch de saint-cierges 3", Some("bercher"));
        assert_eq!(mat.street.unwrap(), "chemin de saint-cierges 3".to_string());
    }

    #[test]
    fn street_matcher_without_place() {
        let mat = StreetMatcher::default().match_by_place("ch de saint-cierges 3", None);
        assert_eq!(mat.street.unwrap(), "chemin de saint-cierges 3".to_string());
    }

    #[test]
    fn street_matcher_with_plz() {
        let mat = StreetMatcher::default().match_by_plz("qu du seujet 36", Some(1201));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(
            mat.file_found.unwrap(),
            PathBuf::from("./test_data/plzs/1201")
        );
    }

    #[test]
    fn street_matcher_without_plz() {
        let mat = StreetMatcher::default().match_by_plz("qu du seujet 36", None);
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    fn street_matcher_wrong_plz() {
        let mat = StreetMatcher::default().match_by_plz("qu du seujet 36", Some(1231231));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    fn street_matcher_wrong_first_word() {
        let mat = StreetMatcher::default().match_by_plz("u du seujet 36", Some(1201));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(
            mat.file_found.unwrap(),
            PathBuf::from("./test_data/plzs/1201")
        )
    }

    #[test]
    fn street_matcher_wrong_first_word_no_plz() {
        let mat = StreetMatcher::default().match_by_plz("u du seujet 36", None);
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    fn street_matcher_wrong_first_word_wrong_plz() {
        let mat = StreetMatcher::default().match_by_plz("u du seujet 36", Some(2132131));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    fn text_matcher_high_sensitivity() {
        let matches = TextMatcher::new(0.99, 5)
            .find_matches_in_file(
                "qu du seujet 36",
                &PathBuf::from_str(DATA_FILE).unwrap(),
                Some(true),
            )
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn text_matcher_zero_to_keep() {
        let matches = TextMatcher::new(0.7, 0)
            .find_matches_in_file(
                "qu du seujet 36",
                &PathBuf::from_str(DATA_FILE).unwrap(),
                Some(true),
            )
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn text_matcher_nomal_sensitivity() {
        let matcher = TextMatcher::new(0.7, 5);
        let matches = matcher
            .find_matches_in_file(
                "qu du seujet 36",
                &PathBuf::from_str(DATA_FILE).unwrap(),
                Some(true),
            )
            .unwrap();
        assert_eq!(matches[0].text, "quai du seujet 36".to_string());
        assert!(matches[0].similarity > 0.7);
    }

    #[test]
    fn text_matcher_find_in_dir() {
        let matches = TextMatcher::find_matches_in_dir(
            0.1,
            5,
            "qu du seujet 36",
            &PathBuf::from_str(DATA_DIR).unwrap(),
            Some(4),
            Some(true),
        );
        assert_eq!(matches[0].text, "quai du seujet 36".to_string());
        assert!(matches[0].similarity > 0.7);
    }
}
