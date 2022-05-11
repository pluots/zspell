use std::collections::LinkedList;

macro_rules! ll_to_vec {
    ($ll:expr,$type:ty) => {{
        $ll.iter().cloned().collect::<Vec<$type>>()
    }};
}

/// Runs ngram() on an input string and collect the resulting tokens into a
/// Vec<String>.
///
/// This macro takes the same arguments as ['ngram'], with the exception thart
/// arg[0] should be a String or &str (anything that implements ['chars']).
///
/// # Examples
///
/// ```
/// use textdistance::str_ngram_vec;
/// let vv = str_ngram_vec!("abcdef", 4, 4);
/// assert_eq!(vv, ["abcd", "bcde", "cdef"]);
/// ```
#[macro_export]
macro_rules! str_ngram_vec {
    ($a:expr,$b:expr,$c:expr) => {{
        $crate::collectors::ngram($a.chars(), $b, $c)
            .map(|x| x.iter().collect::<String>())
            .collect::<Vec<String>>()
    }};
}

/// An [`Iterator`] implementation for calculating a n-gram
#[derive(Debug)]
pub struct NGram<I>
where
    I: Iterator,
{
    iter: I,
    window: LinkedList<I::Item>,
    min: usize,
    max: usize,
}

impl<TY, I> Iterator for NGram<I>
where
    TY: Clone,
    I: Iterator<Item = TY>,
{
    type Item = Vec<I::Item>;

    #[inline]
    fn next(&mut self) -> Option<Vec<I::Item>> {
        match self.iter.next() {
            None => {
                // We've hit the end of the road. Start removing elements, return
                // if we still hvae enough. If not, end of the road for us too.
                self.window.pop_front();

                if self.window.len() >= self.min {
                    Some(ll_to_vec!(self.window, TY))
                } else {
                    None
                }
            }
            Some(next) => {
                self.window.push_back(next);

                // If we're just getting started, fill up our window
                while self.window.len() < self.min {
                    match self.iter.next() {
                        None => return None,
                        Some(v) => self.window.push_back(v),
                    }
                }

                if self.window.len() > self.max {
                    self.window.pop_front();
                }

                Some(ll_to_vec!(self.window, TY))
            }
        }
    }
}

/// Calculate an ngram on any iterable
///
/// When provided
///
/// [Wikipedia](https://en.wikipedia.org/wiki/N-gram) really says it best:
///
/// > In the fields of computational linguistics and probability, an n-gram
/// > (sometimes also called Q-gram) is a contiguous sequence of n items from a
/// > given sample of text or speech. The items can be phonemes, syllables,
/// > letters, words or base pairs according to the application. The n-grams
/// > typically are collected from a text or speech corpus. When the items are
/// > words, n-grams may also be called shingles
///
///
/// # Panics
///
/// Panics if [`min`] is not less than [`max`], or if [`min`] is 0
///
/// # Examples
///
/// Using an
///
/// ```
/// use textdistance::collectors::ngram;
/// let arr_iter = ["yum", "apple", "pie", "woo"].iter();
/// let results = ngram(arr_iter, 2, 3).collect::<Vec<Vec<&str>>>();
/// let expected: Vec<Vec<&str>> = Vec::new();
/// expected.push(vec!["yum", "apple"]);
/// expected.push(vec!["yum", "apple", "pie"]);
/// expected.push(vec!["apple", "pie", "woo"]);
/// expected.push(vec!["pie", "woo"]);
/// assert_eq!(expected, results);
/// ```
pub fn ngram<I>(iter: I, min: usize, max: usize) -> NGram<I>
where
    I: Iterator + Clone,
{
    assert!(
        min <= max,
        "Min <= Max condition not satisfied, {} > {}",
        min,
        max
    );
    assert!(min > 0, "Min must be > 0, {} given", min);

    NGram {
        iter: iter,
        window: LinkedList::new(),
        min: min,
        max: max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Min must be > 0, 0 given")]
    fn test_invalid_minimum() {
        // suppress traceback
        std::panic::set_hook(Box::new(|_| {}));
        str_ngram_vec!("abcdef", 0, 4);
    }

    #[test]
    #[should_panic(expected = "Min <= Max condition not satisfied, 5 > 4")]
    fn test_min_gt_max() {
        // suppress traceback
        std::panic::set_hook(Box::new(|_| {}));
        str_ngram_vec!("abcdef", 5, 4);
    }

    #[test]
    fn test_ngram_empty() {
        let vv: Vec<String> = str_ngram_vec!("", 2, 2);
        let arr: [String; 0] = [];
        assert_eq!(vv, arr);
    }

    #[test]
    fn test_ngram_size_eq() {
        let vv = str_ngram_vec!("abcdef", 2, 2);
        assert_eq!(vv, ["ab", "bc", "cd", "de", "ef"]);
    }

    #[test]
    fn test_ngram_size_1_2() {
        let vv = str_ngram_vec!("abcdef", 1, 2);
        assert_eq!(vv, ["a", "ab", "bc", "cd", "de", "ef", "f"]);
    }

    #[test]
    fn test_ngram_size_range() {
        let vv = str_ngram_vec!("abcdef", 2, 4);
        assert_eq!(vv, ["ab", "abc", "abcd", "bcde", "cdef", "def", "ef"]);
    }

    #[test]
    fn test_ngram_large_size_eq() {
        let vv = str_ngram_vec!("abcdef", 4, 4);
        assert_eq!(vv, ["abcd", "bcde", "cdef"]);
    }

    #[test]
    fn test_ngram_larger_than_arr() {
        let vv = str_ngram_vec!("abcd", 5, 5);
        let arr: [String; 0] = [];
        assert_eq!(vv, arr);
    }

    #[test]
    fn test_ngram_string_arrr() {
        let arr_iter = ["yum", "apple", "pie", "woo"].iter();
        let results = ngram(arr_iter, 2, 3).collect::<Vec<Vec<&str>>>();
        let expected: Vec<Vec<&str>> = Vec::new();
        expected.push(vec!["yum", "apple"]);
        expected.push(vec!["yum", "apple", "pie"]);
        expected.push(vec!["apple", "pie", "woo"]);
        expected.push(vec!["pie", "woo"]);
        assert_eq!(expected, results);
    }
}
