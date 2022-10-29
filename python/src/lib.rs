use pyo3::prelude::*;
use text_matcher_rs::{TextMatcher, Candidate};

#[pymodule]
fn py_text_matcher_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(find_matches_in_dir))?;
    m.add_class::<Candidate>()?;
    m.add_class::<TextMatcher>()?;
    Ok(())
}
