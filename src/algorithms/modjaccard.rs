use std::collections::HashSet;
use std::hash::Hash;

/// Calculate the Jaccard index on two [`HashSet`]s.
///
/// Returns the mathematical Jaccard index, i.e. `|A ∩ B| / |A ∪ B|`
///
/// Usually this is interfaced via [`jaccard`]; that is recommended unless your
/// data is already in a HashSet.
///
/// # Example
///
/// ```
/// use std::collections::HashSet;
/// use stringmetrics::algorithms::jaccard_set;
///
/// let crew1 = HashSet::from(["Einar", "Olaf", "Harald"]);
/// let crew2 = HashSet::from(["Olaf", "Harald", "Birger"]);
///
/// assert_eq!(jaccard_set(crew1, crew2), 0.5);
///
/// ```
///
/// [`HashSet`]: std::collections::HashMap
/// [`jaccard`]: crate::algorithms::jaccard
pub fn jaccard_set<T>(a: HashSet<T>, b: HashSet<T>) -> f32
where
    T: Eq + Hash,
{
    let ii = a.intersection(&b).count();
    let uu = a.union(&b).count();
    ii as f32 / uu as f32
}

/// Calculate the Jaccard index on two iterators using [`jaccard_set`]
///
/// Returns the mathematical Jaccard index, i.e. `|A ∩ B| / |A ∪ B|`. Iterators
/// can point to anything hashable. Often this is combined with an iterator
/// adapter such as [`std::str::Split`] and/or [`core::slice::Windows`] to
/// generate n-grams for text similarity. See [this wikipedia
/// page](https://en.wikipedia.org/wiki/N-gram) for descriptions on n-grams.
///
/// Note: If the data are interested in is already in a HashSet, use
/// [`jaccard_set`] to save the collection step.
///
/// # Example
///
/// ```
/// use stringmetrics::algorithms::jaccard;
///
/// let crew1 = ["Einar", "Olaf", "Harald"];
/// let crew2 = ["Olaf", "Harald", "Birger"];
///
/// assert_eq!(jaccard(crew1.iter(), crew2.iter()), 0.5);
///
/// ```
///
/// Example using using 2-grams. See
/// [this execllent reference](https://www.cs.utah.edu/~jeffp/teaching/cs5140-S15/cs5140/L4-Jaccard+nGram.pdf)
/// for an in-depth explanation of Jaccard Index for k-grams/n-grams.
///
/// ```
/// use stringmetrics::algorithms::jaccard;
///
/// let a = [["to", "be"], ["be", "or"], ["or", "not"]];
/// let b = [["who", "wants"], ["wants", "to"], ["to", "be"]];
///
/// assert_eq!(jaccard(a.iter(), b.iter()), 0.2);
///
/// ```
///
pub fn jaccard<T, U>(a: T, b: T) -> f32
where
    T: Iterator<Item = U>,
    U: Hash + Eq,
{
    let aa: HashSet<U> = a.collect();
    let bb: HashSet<U> = b.collect();
    jaccard_set(aa, bb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_empty() {
        assert!(jaccard("".chars(), "".chars()).is_nan());
    }

    #[test]
    fn test_jaccard_a_empty() {
        assert_eq!(jaccard("".chars(), "ab".chars()), 0f32);
    }

    #[test]
    fn test_jaccard_b_empty() {
        assert_eq!(jaccard("ab".chars(), "".chars()), 0f32);
    }

    #[test]
    fn test_jaccard_str_sets() {
        let a = ['a', 'b', 'c'].iter();
        let b = ['b', 'c', 'd'].iter();

        assert_eq!(jaccard(a, b), 0.5);
    }
}
