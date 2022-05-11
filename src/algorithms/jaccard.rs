use std::collections::HashSet;
use std::hash::Hash;
// use itertools::{chain, Itertools};

// #[inline]
// fn get_unique_tokens (s: &str, ngram: usize) ->Unique<Windows<T>>{
//     let a =s.chars().collect::<Vec<char>>().windows(ngram).into_iter().unique();
// }

// Accept an iterator
// pub fn jaccard_window_size(a: &str, b: &str, ngram: usize) -> f32 {
//     // Turn our strings into vectors
//     let a_token_iter = a.chars().windows(ngram).into_iter().unique();
//     let b_token_iter = b.chars().windows(ngram).into_iter().unique();

//     // s
//     chain = a_token_iter.chain(b_token_iter);
//     let total = chain;

//     chain_unique = chain.unique();
//     let unique_count = chain_unique.len();

//     let mut unions = 0u32;
//     let mut intersects = 0u32;

//     0f32
// }

// Accept two iterators
// pub fn jaccard(a: &str, b: &str) -> f32 {
//     jaccard_window_size(a, b, 2)
// }

pub fn jaccard<T: Hash + Eq>(a: HashSet<T>, b: HashSet<T>) -> f32 {
    let ii = a.intersection(&b).count();
    let uu = a.union(&b).count();
    ii as f32 / uu as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_empty() {
        // assert_eq!(jaccard("", ""), 0f32);
    }
}
