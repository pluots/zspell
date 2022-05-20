//! This module contains functions for applying various closeness algorithms.
//!
//! See the indiviual functions for usage and examples. This page intends to
//! give a high level overview of how the functions are implemented.
//!
//! # Levenshtein distance algorithm
//!
//! The funcition [`levenshtein`] implements the algorithm below:
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
//! 

mod basic;
mod damerau;
mod jaccard;
mod levenshtein;

pub use self::basic::hamming;
pub use self::damerau::damerau_levenshtein;
pub use self::jaccard::{jaccard, jaccard_set};
pub use self::levenshtein::{
    levenshtein, levenshtein_limit, levenshtein_limit_weight, levenshtein_weight,
};
