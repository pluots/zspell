// use pyo3::prelude::*;
// // use pyo3::types::{PyUnicode};
// use zspell::;

// /// Formats the sum of two numbers as string.
// #[pyfunction]
// #[inline]
// fn levenshtein(a: &str, b: &str) -> u32 {
//     levenshtein_limit_weight(a, b, u32::MAX, 1, 1, 1)
// }

// #[pyfunction]
// #[inline]
// fn levenshtein_quick(a: String, b: String) -> u32 {
//     modlev_quick(a, b)
// }

// /// Formats the sum of two numbers as string.
// #[pyfunction]
// #[inline]
// fn levenshtein_limit(a: &str, b: &str, limit: u32) -> u32 {
//     levenshtein_limit_weight(a, b, limit, 1, 1, 1)
// }

// /// A Python module implemented in Rust.
// #[pymodule]
// #[inline]
// fn zspell(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(levenshtein, m)?)?;
//     m.add_function(wrap_pyfunction!(levenshtein_limit, m)?)?;
//     m.add_function(wrap_pyfunction!(levenshtein_quick, m)?)?;
//     Ok(())
// }
