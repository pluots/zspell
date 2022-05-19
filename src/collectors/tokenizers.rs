use std::collections::LinkedList;
use std::num::NonZeroUsize;

macro_rules! ll_to_vec {
    ($ll:expr,$type:ty) => {{
        $ll.iter().cloned().collect::<Vec<$type>>()
    }};
}

/// Runs ngrams() on an input string and collect the resulting tokens into a
/// Vec<String>.
///
/// This macro takes the same arguments as ['ngrams'], with the exception thart
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
        $crate::collectors::ngrams($a.chars(), $b, $c)
            .map(|x| x.iter().collect::<String>())
            .collect::<Vec<String>>()
    }};
}

/// An [`Iterator`] implementation for calculating a n-gram
/// 
/// This is created by the [`ngrams`] method on any iterable
///
/// # Example
///
/// ```
/// let mychars = ['r', 'u', 's', 't'];
/// let iter = mychars.ngrams(2, 3);
/// ```
///
/// [`windows`]: crate::windows
#[derive(Debug)]
pub struct NGrams<'a, T: 'a>
{
    v: &'a [T],
    min: NonZeroUsize,
    max: NonZeroUsize,
    wsize: usize, // Working variable to hold current window size
}

impl<'a, T: 'a> NGrams<'a, T> {
    #[inline]
    pub(super) fn new(slice: &'a [T], min: NonZeroUsize, max: NonZeroUsize) -> Self {
        Self { v: slice, min,max ,wsize:min.get()}
    }
}


impl<'a, T> Iterator for NGrams<'a,T>
{
    // type Item = Vec<I::Item>;
    type Item = &'a [T];

    #[inline]
    // fn next(&mut self) -> Option<Vec<I::Item>> {
    fn next(&mut self) -> Option<&'a [T]> {
        let x = (1..2).map()
        if self.min.get() > self.v.len() {
            return None;
        }
        let retval = Some(&self.v[..self.wsize]);
        
        if self.wsize < self.max.get() {
            self.wsize+=1;
        } else {
            self.v = &self.v[1..];
        }

        retval
    }
}

/// Calculate an ngrams on any iterable
///
/// This is an iterator adapter that accepts any iterator and returns an
/// iterator of its ngrams. Regarding what a ngrams
/// is, [Wikipedia](https://en.wikipedia.org/wiki/N-gram) really says it best:
///
/// > In the fields of computational linguistics and probability, an n-gram
/// > (sometimes also called Q-gram) is a contiguous sequence of n items from a
/// > given sample of text or speech. The items can be phonemes, syllables,
/// > letters, words or base pairs according to the application. The n-grams
/// > typically are collected from a text or speech corpus. When the items are
/// > words, n-grams may also be called shingles
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
/// use textdistance::collectors::ngrams;
/// let arr_iter = ["yum", "apple", "pie", "woo"].iter();
/// let results = ngrams(arr_iter, 2, 3).collect::<Vec<Vec<&str>>>();
/// 
/// let expected: Vec<Vec<&str>> = Vec::new();
/// expected.push(vec!["yum", "apple"]);
/// expected.push(vec!["yum", "apple", "pie"]);
/// expected.push(vec!["apple", "pie", "woo"]);
/// expected.push(vec!["pie", "woo"]);
/// assert_eq!(expected, results);
/// ```
/// 
#[inline]
pub fn ngrams(&self, min: usize, max: usize) -> NGrams<'_,T> {
    let min=NonZeroUsize::new(min).expect("min is zero");
    let max=NonZeroUsize::new(max).expect("max is zero");
    assert!(
        min <= max,
        "Min <= Max condition not satisfied, {} > {}",
        min,
        max
    );

    NGrams::new(self,min,max)
}

// #![feature(type_name_of_val)]
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
        // let vv 
        // str_ngram_vec!("", 2, 2);
        let arr: [String; 0] = [];
        assert_eq!(vv, arr);
    }

    #[test]
    fn test_ngram_size_eq() {
        // let vv = str_ngram_vec!("abcdef", 2, 2);
        let s = "abcdef";
        let vv = s.chars().ngrams(2,2).collect();
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

    fn print_type_of<T>(_: &T) {
        println!("{}", std::any::type_name::<T>())
    }

    #[test]
    fn test_ngram_string_arrr() {
        let arr_iter = ["yum", "apple", "pie", "woo"].iter();
        let xx = ngrams(arr_iter, 2, 3);
        let xx = ["yum", "apple", "pie", "woo"].windows(3);
        let results = ngrams(arr_iter, 2, 3).collect::<Vec<Vec<&&str>>>();
        let mut expected = Vec::new();
        expected.push(vec!["yum", "apple"]);
        expected.push(vec!["yum", "apple", "pie"]);
        expected.push(vec!["apple", "pie", "woo"]);
        expected.push(vec!["pie", "woo"]);
        println!("{:?}",results);
        print_type_of(&results);
        println!("{:?}",expected);
        print_type_of(&expected);
        // assert_eq!(expected, results);
    }
}
