use std::cmp::{max, min};

/// Main algorithm comes from the Levenshtein Distance wikipedia page
pub fn levenshtein_limit_weight(
    a: &str,
    b: &str,
    limit: u32,
    ins_cost: u32,
    del_cost: u32,
    sub_cost: u32,
) -> u32 {
    let mut limit = limit;
    if limit == 0 {
        limit = u32::MAX;
    }
    let a_len = a.len();
    let b_len = b.len();
    let diff = (max(a_len, b_len) - min(a_len, b_len)) as u32;

    if diff >= limit {
        return limit;
    }

    // Create two working vectors
    let v_len = a.len() + 1;
    let mut v_prev: Vec<u32> = (0..(v_len as u32)).collect();
    let mut v_curr: Vec<u32> = vec![0; v_len];

    let mut deletion_cost: u32;
    let mut insertion_cost: u32;
    let mut substitution_cost: u32;

    for (i, b_char) in b.chars().enumerate() {
        v_curr[0] = (i + 1) as u32;

        // Fill out the rest of the row
        for (j, a_char) in a.chars().enumerate() {
            // calculating costs for A[i+1][j+1]
            deletion_cost = (v_prev[j + 1] + 1) * del_cost;
            insertion_cost = (v_curr[j] + 1) * ins_cost;
            substitution_cost = (match a_char == b_char {
                true => v_prev[j],
                false => v_prev[j] + 1,
            }) * sub_cost;

            v_curr[j + 1] = min(min(deletion_cost, insertion_cost), substitution_cost);
        }
        let current_max = v_curr.last().copied().unwrap_or_default();
        if current_max >= limit {
            return limit;
        }

        // Move current row to previous for the next loop
        // "Current" is always overwritten so we can just swap
        std::mem::swap(&mut v_prev, &mut v_curr);
    }

    // Remember we swapped
    v_prev.last().copied().unwrap_or_default()
}

/// Levenshtein distance computation with a limit
///
/// This will limitate the levshtein distance to a given maximum value
///
/// ```
/// use textdistance::algorithms::levenshtein_limit;
/// let a = "abcdefg";
/// let b = "mmmmmmm";
/// assert_eq!(levenshtein_limit(a, b, 3), 3);
/// ```
pub fn levenshtein_limit(a: &str, b: &str, limit: u32) -> u32 {
    levenshtein_limit_weight(a, b, limit, 1, 1, 1)
}

/// Levenshtein distance computation
///
/// ```
/// use textdistance::algorithms::levenshtein;
/// let a = "this is a book";
/// let b = "i am a cook";
/// assert_eq!(levenshtein(a, b), 6);
/// ```
pub fn levenshtein(a: &str, b: &str) -> u32 {
    levenshtein_limit_weight(a, b, 0, 1, 1, 1)
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
    fn test_levenshtein_only_a() {
        assert_eq!(levenshtein("abcdef", ""), 6);
    }

    #[test]
    fn test_levenshtein_only_b() {
        assert_eq!(levenshtein("", "abcdef"), 6);
    }

    #[test]
    fn test_levenshtein_basic() {
        assert_eq!(levenshtein("abcdef", "abdde"), 2);
    }

    #[test]
    fn test_levenshtein_limit_one_empty() {
        assert_eq!(levenshtein_limit("abcdef", "", 3), 3);
    }

    #[test]
    fn test_levenshtein_limit() {
        assert_eq!(levenshtein_limit("abcdef", "ghijkl", 3), 3);
    }
}
