mod basic;
mod levenshtein;
pub use self::basic::hamming;
pub use self::levenshtein::levenshtein;
pub use self::levenshtein::levenshtein_trunc;
