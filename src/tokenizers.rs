//! Tokenizers just provide handy methods to split words up following rules

// Characters to be removed on all occasions
const REMOVE_CHARS: [char; 10] = ['(', ')', ',', '\"', '.', ';', ':', '?', '[', ']'];
// Remove these from the ends only
const END_REMOVE_CHARS: [char; 2] = ['-', '\''];

/// Split by whitespace an remove all punctuation
///
/// Standard spellcheck tokenizer
#[inline]
pub fn split_whitespace_remove_punc(s: &str) -> impl Iterator<Item = String> + '_ {
    // TODO: benchmark whether it is faster to replace first, or at the end
    // Note that we leave the "'" since it's useful for apostrophe
    s.split_whitespace()
        .map(|word| {
            word.trim_matches(&END_REMOVE_CHARS[..])
                .replace(&REMOVE_CHARS, "")
        })
        .filter(|word| !word.is_empty())
}
