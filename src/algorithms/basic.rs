/// Hamming distance computation
///
/// ```
/// use textdistance::algorithms::hamming;
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
