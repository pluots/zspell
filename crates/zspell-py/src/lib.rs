//! Wrappers around the `zspell` module to expose it to Python

use ::zspell as z;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug)]
/// This is the main dictionary interface.
///
/// To use it, you need to load in both an affix configuration file and a
/// dictionary file. Sometimes these are installed on your system but if not,
/// this repository has them available:
/// <https://github.com/wooorm/dictionaries>.
///
/// ```pycon
/// >>> from zspell import Dictionary
/// >>> with open ("dictionaries/en_US.aff", "r") as f:
/// ...     config_str = f.read()
/// ...
/// >>> with open ("dictionaries/en_US.dic", "r") as f:
/// ...     dict_str = f.read()
/// ...
/// >>> d = Dictionary(config_str, dict_str)
/// >>> d.check("Apples are good! Don't you think?")
/// True
/// >>> d.check("Apples are baaaad")
/// False
/// ```
#[pyo3(text_signature = "(config_str, dict_str)")]
struct Dictionary(z::Dictionary);

#[pymethods]
impl Dictionary {
    /// Create a new dictionary
    #[new]
    fn new(config_str: &str, dict_str: &str, personal_str: Option<&str>) -> PyResult<Self> {
        let mut builder = z::DictBuilder::new()
            .dict_str(dict_str)
            .config_str(config_str);

        if let Some(personal) = personal_str {
            builder = builder.personal_str(personal);
        }

        match builder.build() {
            Ok(dict) => Ok(Self(dict)),
            Err(err) => Err(convert_error(err)),
        }
    }

    /// Check if a string is valid.
    #[pyo3(text_signature = "($self, input)")]
    fn check(&self, input: &str) -> bool {
        self.0.check(input)
    }

    /// Check if a single word is valid.
    #[pyo3(text_signature = "($self, word)")]
    fn check_word(&self, word: &str) -> bool {
        self.0.check_word(word)
    }
    // TODO: figure out how to convert to a python iterator
    // fn check_indices<'a: 'd, 'd>(&'d self, word: &'a str) -> impl Iterator<Item =  (usize, &'a str)> + 'd{
    //     self.0.check_indices(word)
    // }
}

fn convert_error(err: z::Error) -> PyErr {
    match err {
        z::Error::Parse(e) => ParseError::new_err(format!("{e}")),
        z::Error::Build(e) => BuildError::new_err(format!("{e}")),
        z::Error::Regex(e) => RegexError::new_err(format!("{e}")),
        z::Error::Io(e) => IoError::new_err(format!("{e}")),
        _ => unreachable!(),
    }
}

create_exception!(
    my_module,
    BuildError,
    PyException,
    "Raised when there is an error building the dictionary."
);
create_exception!(
    my_module,
    ParseError,
    PyException,
    "Raised when there is an error parsing dictionary input."
);
create_exception!(
    my_module,
    RegexError,
    PyException,
    "Raised when there is an error with parsed regex."
);
create_exception!(
    my_module,
    IoError,
    PyException,
    "Raised when there is an I/O error."
);

#[pymodule]
fn zspell(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Dictionary>()?;
    m.add("BuildError", py.get_type::<BuildError>())?;
    m.add("ParseError", py.get_type::<ParseError>())?;
    m.add("IoError", py.get_type::<IoError>())?;
    m.add("RegexError", py.get_type::<RegexError>())?;
    Ok(())
}
