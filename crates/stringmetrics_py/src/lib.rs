use pyo3::prelude::*;
use stringmetrics::algorithms::levenshtein as alg_levenshtein;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn levenshtein(a: &str, b: &str) -> PyResult<usize> {
    Ok(alg_levenshtein(a, b) as usize)
}

/// A Python module implemented in Rust.
#[pymodule]
fn stringmetrics(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(levenshtein, m)?)?;
    Ok(())
}
