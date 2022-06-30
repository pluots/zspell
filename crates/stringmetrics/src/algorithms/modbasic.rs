/// Hamming distance computation
///
/// The hamming distance between two equal length strings is the number of
/// substitutions required to change one string into the other. This function
/// calculates that.
///
/// # Panics
///
/// If the given strings are not equal length, this function will panic
///
/// # Example
///
/// ```
/// use stringmetrics::algorithms::hamming;
/// let a = "abcdefg";
/// let b = "aaadefa";
/// assert_eq!(hamming(a, b), 3);
/// ```
pub fn hamming(a: &str, b: &str) -> u32 {
    assert_eq!(
        a.len(),
        b.len(),
        "Hamming distance requires slices of equal length, lengths {} and {} given",
        a.len(),
        b.len()
    );

    let mut distance = 0;

    let zipped = a.chars().zip(b.chars());

    for (aa, bb) in zipped {
        if aa != bb {
            distance += 1;
        }
    }

    distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_panic_on_not_equal() {
        hamming("abc", "ab");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(hamming("", ""), 0);
    }

    #[test]
    fn test_basic() {
        assert_eq!(hamming("abcdefg", "0bc1ef2"), 3);
    }
}
