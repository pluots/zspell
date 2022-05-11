use std::collections::LinkedList;

macro_rules! ll_to_vec {
    ($ll:expr,$type:ty) => {{
        $ll.iter().cloned().collect::<Vec<$type>>()
    }};
}

/// Run ngram() on a string and collect the results into a Vec<String>
#[macro_export]
macro_rules! str_ngram_vec {
    ($a:expr,$b:expr,$c:expr) => {{
        ngram($a.chars(), $b, $c)
            .map(|x| x.iter().collect::<String>())
            .collect::<Vec<String>>()
    }};
}

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

/// Supply an iterator of any type and this will return a windowed token
/// iterator.
pub fn ngram<I>(iter: I, min: usize, max: usize) -> NGram<I>
where
    I: Iterator + Clone,
{
    assert!(min <= max, "Min <= Max condition not satisfied, {} > {}",min,max);
    assert!(min > 0, "Min must be > 0, {} given",min);

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
        assert_eq!(vv, ["a", "ab","bc", "cd", "de", "ef","f"]);
    }

    #[test]
    fn test_ngram_size_range() {
        let vv = str_ngram_vec!("abcdef", 2, 4);
        assert_eq!(vv, ["ab", "abc", "abcd", "bcde", "cdef","def","ef"]);
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
}
