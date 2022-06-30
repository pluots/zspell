//! This module contains functions for applying various closeness algorithms.
//!
//! See the indiviual functions for usage and examples. This page intends to
//! give a high level overview of how the functions are implemented.
//!
//! # Levenshtein distance algorithm
//!
//! The funcition [levenshtein][crate::algorithms::levenshtein] implements the
//! algorithm below:
//!
//! $$ \operatorname{lev}_{a,b}(i,j) = \begin{cases} \max(i,j) &\text{if }
//! \min(i,j) = 0, \\
//!    \min \begin{cases} \operatorname{lev}_{a,b}(i-1,j) + 1 \\
//!         \operatorname{lev}_{a,b}(i,j-1) + 1 \\
//!         \operatorname{lev}_{a,b}(i-1,j-1) + 1_{(a_i \ne b_i)}
//!       \end{cases}
//!    &\text{otherwise} \end{cases}$$
//!
//! _(erm... I can't seem to get KaTeX working. Let me know on GitHub if you can
//! help!)_
//!
//! Easier shown than read. Basically, the algorithm parses from top left to
//! bottom right to create a table like follows for the classic example:
//!
//! ```text
//!      j → 0  1  2  3  4  5  6  7
//! i             str B ->
//! ↓           s  i  t  t  i  n  g
//! 0  s    [0, 1, 2, 3, 4, 5, 6, 7]
//! 1  t  k [1, 1, 2, 3, 4, 5, 6, 7]
//! 2  r  i [2, 2, 1, 2, 3, 4, 5, 6]
//! 3     t [3, 3, 2, 1, 2, 3, 4, 5]
//! 4  A  t [4, 4, 3, 2, 1, 2, 3, 4]
//! 5  ↓  e [5, 5, 4, 3, 2, 2, 3, 4]
//! 6     n [6, 6, 5, 4, 3, 3, 2, 3]
//! ```
//!
//! The outer rows (at i=0 and j=0) are just incrementing and can be filled in
//! automatically. Then, the algorithm works its way left-to-right then
//! top-to-bottom to fill in the table. Rules are:
//!
//! 1. Insertion cost is the value of the field to the left plus one
//! 2. Deletion cost is the value of the field above plus one
//! 3. Substitution cost is the value of the field left above if the two
//!    relevant characters are the same. If they are different, add one to this
//!    value.
//! 4. Take the minimum of these three values and put it in the current box.
//!
//! Eventually, the above matrix can be filled out and the final "distance" is
//! the number at the bottom right; in this case, 3.
//!
//! This library uses [an algorithm published by Sten
//! Hjelmqvist](https://www.codeproject.com/Articles/13525/Fast-memory-efficient-Levenshtein-algorithm-2)
//! and described on [the Levenshtein distance Wikipedia
//! page](https://en.wikipedia.org/wiki/Levenshtein_distance#Iterative_with_two_matrix_rows)
//! that only uses two vectors at a time, rather than constructing the entire
//! matrix, for memory optimizations. Memory use is only that for a vector of
//! u32 twice the length of string B.
//!
//! ## Limited Levenshtein algorithm
//!
//! This is easy; same algorithm as above, just stop matching when you hit a
//! desired limit to avoid spending resources on obviously different strings.
//! Use this version where possible, implemented by [`levenshtein_limit`].
//!
//! ```
//! use stringmetrics::algorithms::levenshtein_limit;
//! assert_eq!(levenshtein_limit("superlongstring", "", 3), 3);
//! ```
//!
//! ## Weighted Levenshtein algorithm
//!
//! Implemented by [`levenshtein_weight`] and [`levenshtein_limit_weight`], a
//! weighted Levenshtein algorithm just allows applying differing penalties for
//! insertion, deletion, and substitution. It's basically the same algorithm as
//! above, except the added values are weights rather than one, and the initial
//! row population is related to insertion and deletion weights.
//!
//! For example, the following code:
//!
//! ```
//! use stringmetrics::algorithms::levenshtein_weight;
//! assert_eq!(levenshtein_weight("kitten", "sitting", 4, 3, 2), 8);
//! ```
//! Creates the following matrix:
//!
//! ```text
//!       j → 0   1   2   3   4   5   6   7
//! i             str B ->
//! ↓             s   i   t   t   i   n   g
//! 0  s    [ 0,  4,  8, 12, 16, 20, 24, 28]
//! 1  t  k [ 3,  2,  6, 10, 14, 18, 22, 26]
//! 2  r  i [ 6,  5,  2,  6, 10, 14, 18, 22]
//! 3     t [ 9,  8,  5,  2,  6, 10, 14, 18]
//! 4  A  t [12, 11,  8,  5,  2,  6, 10, 14]
//! 5  ↓  e [15, 14, 11,  8,  5,  4,  8, 12]
//! 6     n [18, 17, 14, 11,  8,  7,  4,  8]
//! ```
//!
//! The result of 8 is representative of one added letter (+4) and two
//! substitutions (+2*2). The substitution could alternatively be counted as an
//! insertion followed by a deletion but the algorithm "chooses" against it
//! since the cost would be much higher (4+3=7 when the substitution cost is
//! only 2).)

use std::cmp::{max, min};

use unicode_segmentation::UnicodeSegmentation;

/// Basic Levenshtein distance computation
///
/// This runs the levenshtein distance algorithm on all strings with all costs
/// equal to 1 and with no limits, which is suitable for cases where an exact
/// distance is needed. Use cases are usually those where the strings are known
/// to not be "very different" (e.g., strings have similar lengths). In most
/// cases it is better to use [`levenshtein_limit`] to avoid unnecessary
/// computation.
///
/// Behind the scenes, this wraps [`levenshtein_limit_weight`]. For details on
/// operation, see the [algorithms](crate::algorithms) page.
///
/// # Example
///
/// ```
/// use stringmetrics::algorithms::levenshtein;
/// let a = "this is a book";
/// let b = "i am a cook";
/// assert_eq!(levenshtein(a, b), 6);
/// ```
///
/// Note that sometimes the levenshtein distance is defined as having a default
/// weight of 2 for substitutions. That isn't the case for this implementation -
/// if you need that functionality, please use [`levenshtein_weight`].
#[inline]
pub fn levenshtein(a: &str, b: &str) -> u32 {
    levenshtein_limit_weight(a, b, u32::MAX, 1, 1, 1)
}

/// Levenshtein distance computation with a limit
///
/// This will limitate the levshtein distance up to a given maximum value. The
/// usual reason for wanting to do this is to avoid unnecessary computation when
/// a match between two strings can quickly be pruned as "different".
///
/// Behind the scenes, this wraps [`levenshtein_limit_weight`].
///
/// # Example
///
/// ```
/// use stringmetrics::algorithms::levenshtein_limit;
/// let a = "abcdefg";
/// let b = "mmmmmmm";
/// assert_eq!(levenshtein_limit(a, b, 3), 3);
/// ```
///
#[inline]
pub fn levenshtein_limit(a: &str, b: &str, limit: u32) -> u32 {
    levenshtein_limit_weight(a, b, limit, 1, 1, 1)
}

/// Levenshtein distance computation with weights
///
/// Allows setting costs for inserts, deletes and substitutions. See
/// [algorithms](crate::algorithms) for details on weight computation.
///
/// Behind the scenes, this wraps [`levenshtein_limit_weight`].
///
/// # Example
///
/// In this example, an insertion weight of 4, deletion weight of 3, and
/// substitution weight of 2 are used.
///
/// ```
/// use stringmetrics::algorithms::levenshtein_weight;
/// assert_eq!(levenshtein_weight("kitten", "sitting", 4, 3, 2), 8);
/// ```
#[inline]
pub fn levenshtein_weight(a: &str, b: &str, w_ins: u32, w_del: u32, w_sub: u32) -> u32 {
    levenshtein_limit_weight(a, b, u32::MAX, w_ins, w_del, w_sub)
}

/// Levenshtein distance computations with adjustable weights and a limit
///
/// This function implements calculation of the [levenshtein
/// distance](https://en.wikipedia.org/wiki/Levenshtein_distance) between two
/// strings, with specified costs for insertion, deletion, and substitution, and
/// a limit. The other non-iterator functions in this module simply wrap it, and
/// it's generally easier to use any of those (e.g. [`levenshtein_limit`])
/// unless you need all the functionality that this has to offer.
///
/// This function accepts two strings, which are then split on `.graphemes`
/// rather than `.chars`. This ensures that multibyte UTF sequences come with
/// the expected results.
///
/// Note that this algorithm does not (yet) apply any sort of per-character
/// weights, as some implementations may allow for. Instead, it assumes that all
/// substitutions have a cost of 0 if the characters are equal, and the
/// specified weight if the characters are not equal.
///
/// See [algorithms](crate::algorithms) for a detailed description of the
/// algorithm in use.
///
///
/// # Example
///
/// In this example, an insertion weight of 4, deletion weight of 3, and
/// substitution weight of 2 are used. A limit of 6 is applied, and we see that
/// we hit that limit.
///
/// ```
/// use stringmetrics::algorithms::levenshtein_limit_weight;
/// assert_eq!(levenshtein_limit_weight("kitten", "sitting", 6, 4, 3, 2), 6);
/// ```
///
/// With a more reasonable limit, we get a representative result. The 8 comes
/// from one added letter (4) and two substitutions.
///
/// ```
/// use stringmetrics::algorithms::levenshtein_limit_weight;
/// assert_eq!(levenshtein_limit_weight("kitten", "sitting", 100, 4, 3, 2), 8);
/// ```
pub fn levenshtein_limit_weight(
    a: &str,
    b: &str,
    limit: u32,
    w_ins: u32,
    w_del: u32,
    w_sub: u32,
) -> u32 {
    levenshtein_limit_weight_iter(
        a.graphemes(true),
        b.graphemes(true),
        limit,
        w_ins,
        w_del,
        w_sub,
    )
}
/// Levenshthein distance computation on anything iterable that implementes
/// [`PartialEq`].
///
/// This can be used when Levenshthein distance is applicable to something other
/// than strings, or when you wish to iterate on characters rather than
/// graphemes (which is a somewhat rare case).
///
/// # Examples
///
/// An example for the above function, using a 4-byte character and splitting by
/// graphemes:
///
/// ```
/// use stringmetrics::algorithms::levenshtein_limit_weight;
/// assert_eq!(levenshtein_limit_weight("🏴‍☠️", "A", 100, 1, 1, 1), 1);
/// ```
///
/// Using the below function to split on byte ("char") boundaries instead. Note
/// that this is probably something that you'd never actually want to do.
///
/// ```
/// use stringmetrics::algorithms::levenshtein_limit_weight_iter;
/// let iter1 = "🏴‍☠️".chars();   // ['🏴', '\u{200d}', '☠', '\u{fe0f}']
/// let iter2 = "A".chars();    // ['A']
/// assert_eq!(levenshtein_limit_weight_iter(iter1, iter2, 100, 1, 1, 1), 4);
/// ```
pub fn levenshtein_limit_weight_iter<T, I>(
    a: T,
    b: T,
    limit: u32,
    w_ins: u32,
    w_del: u32,
    w_sub: u32,
) -> u32
where
    T: IntoIterator<Item = I>,
    I: PartialEq,
{
    // Need to collect to vectors first so we get a finite length
    let a_vec: Vec<I> = a.into_iter().collect();
    let b_vec: Vec<I> = b.into_iter().collect();

    let a_len = a_vec.len() as u32;
    let b_len = b_vec.len() as u32;

    // Start with some shortcut solution optimizations
    if a_len == 0 {
        return min(b_len * w_ins, limit);
    }
    if b_len == 0 {
        return min(a_len * w_del, limit);
    }

    let diff = max(a_len, b_len) - min(a_len, b_len);
    if diff >= limit {
        return limit;
    }

    // These vectors will hold the "previous" and "active" distance row, rather
    // than needing to construct the entire array. We want to keep these small
    // so a vector of u32 is preferred over usize. u16 would be even better but
    // for long text, that could be hit somewhat easily.
    let v_len = b_len + 1;
    let mut v_prev: Vec<u32> = (0..(v_len * w_ins)).step_by(w_ins as usize).collect();
    let mut v_curr: Vec<u32> = vec![0; v_len as usize];

    let mut ins_cost: u32;
    let mut del_cost: u32;
    let mut sub_cost: u32;
    let mut current_max: u32 = 0;
    // i holds our "vertical" position, j our "horizontal". We fill the table
    // top to bottom. Note there is actually an offset of 1 from i to the "true"
    // array position (since we start one row down).
    for (i, a_item) in a_vec.iter().enumerate() {
        v_curr[0] = ((i + 1) * w_del as usize) as u32;
        // Fill out the rest of the row
        for (j, b_item) in b_vec.iter().enumerate() {
            ins_cost = v_curr[j] + w_ins;
            del_cost = v_prev[j + 1] + w_del;
            sub_cost = match a_item == b_item {
                true => v_prev[j],
                false => v_prev[j] + w_sub,
            };

            v_curr[j + 1] = min(min(ins_cost, del_cost), sub_cost);
        }

        current_max = *v_curr.last().unwrap();

        if current_max >= limit {
            return limit;
        }

        // Move current row to previous for the next loop
        // "Current" is always overwritten so we can just swap
        std::mem::swap(&mut v_prev, &mut v_curr);
    }

    current_max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn test_levenshtein_equal() {
        assert_eq!(levenshtein("abcdef", "abcdef"), 0);
    }

    #[test]
    fn test_levenshtein_one_empty() {
        assert_eq!(levenshtein("abcdef", ""), 6);
        assert_eq!(levenshtein("", "abcdef"), 6);
    }

    #[test]
    fn test_levenshtein_basic() {
        assert_eq!(levenshtein("abcd", "ab"), 2);
        assert_eq!(levenshtein("abcd", "ad"), 2);
        assert_eq!(levenshtein("abcd", "cd"), 2);
        assert_eq!(levenshtein("abcd", "a"), 3);
        assert_eq!(levenshtein("abcd", "c"), 3);
        assert_eq!(levenshtein("abcd", "accd"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("to be a bee", "not to bee"), 6);
    }

    #[test]
    fn test_levenshtein_limit_one_empty() {
        assert_eq!(levenshtein_limit("abcdef", "", 3), 3);
        assert_eq!(levenshtein_limit("", "abcdef", 3), 3);
        assert_eq!(levenshtein_limit("abcdef", "", 8), 6);
        assert_eq!(levenshtein_limit("", "abcdef", 8), 6);
    }

    #[test]
    fn test_levenshtein_limit() {
        // Most of this is tested via levenshtein()
        // just need to validate limits
        assert_eq!(levenshtein_limit("abcdef", "000000", 3), 3);
        assert_eq!(levenshtein_limit("ab", "0000", 3), 3);
    }

    #[test]
    fn test_levenshtein_weight_insertion() {
        assert_eq!(levenshtein_weight("", "a", 10, 1, 1), 10);
        assert_eq!(levenshtein_weight("a", "", 10, 1, 1), 1);
        assert_eq!(levenshtein_weight("", "ab", 10, 1, 1), 20);
        assert_eq!(levenshtein_weight("ab", "", 10, 1, 1), 2);
        assert_eq!(levenshtein_weight("ab", "abcd", 10, 1, 1), 20);
        assert_eq!(levenshtein_weight("kitten", "sitting", 10, 1, 1), 12);
    }

    #[test]
    fn test_levenshtein_weight_deletion() {
        assert_eq!(levenshtein_weight("", "a", 1, 10, 1), 1);
        assert_eq!(levenshtein_weight("a", "", 1, 10, 1), 10);
        assert_eq!(levenshtein_weight("", "ab", 1, 10, 1), 2);
        assert_eq!(levenshtein_weight("ab", "", 1, 10, 1), 20);
        assert_eq!(levenshtein_weight("abc", "ac", 1, 10, 2), 10);
        assert_eq!(levenshtein_weight("abcd", "ac", 1, 10, 2), 20);
        assert_eq!(levenshtein_weight("kitten", "sitting", 1, 10, 1), 3);
    }

    #[test]
    fn test_levenshtein_weight_substitution() {
        // Note that when substitution cost is high, the algorithm will prefer
        // a deletion and insertion
        assert_eq!(levenshtein_weight("a", "b", 10, 10, 5), 5);
        assert_eq!(levenshtein_weight("abcd", "acc", 10, 10, 2), 12);
        assert_eq!(levenshtein_weight("kitten", "sitting", 4, 3, 2), 8);
    }
}
