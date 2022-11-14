//! This crate is aimed to be a simple and fast solution for text-matching from the file with
//! more than 2 millions of lines, especially for streets in Switzerland.
//!
//! Also, it serves as my first Rust project used for work and published out to the people
mod candidate;
mod street_matcher;
mod text_matcher;

pub use candidate::{Candidate, Sensitivity};
use pyo3::prelude::*;
pub use street_matcher::{Place, Plz, Street, StreetMatcher};

#[derive(FromPyObject)]
enum PyLocation<'a> {
    #[pyo3(transparent)]
    Place(&'a str),
    #[pyo3(transparent)]
    Plz(usize),
}

type FoundData<T> = (Option<Candidate>, Option<T>);

#[pyclass]
struct PyCandidate {
    #[pyo3(get)]
    street: Option<String>,
    #[pyo3(get)]
    location: Option<String>,
}

impl PyCandidate {
    fn from<T>(found_data: FoundData<T>) -> Self
    where
        T: ToString,
    {
        PyCandidate {
            street: found_data.0.map(|cand| cand.text),
            location: found_data.1.map(|loc| loc.to_string()),
        }
    }
}

#[pyfunction]
fn find_street(sens: f64, street: &str, loc: Option<PyLocation>) -> PyCandidate {
    let sm = StreetMatcher {
        sens: Sensitivity::new(sens),
    };
    let street = Street::new(street);
    loc.map_or_else(
        || PyCandidate::from(sm.find_matches::<Plz>(&street, None)),
        |loc| match loc {
            PyLocation::Place(place) => {
                PyCandidate::from(sm.find_matches(&street, Some(Place::new(place))))
            }
            PyLocation::Plz(plz) => {
                PyCandidate::from(sm.find_matches(&street, Some(Plz::new(plz))))
            }
        },
    )
}

#[pymodule]
fn text_matcher_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(find_street, m)?)?;
    m.add_class::<PyCandidate>()?;
    Ok(())
}
