//! Token Counting Utilities

use tiktoken_rs::cl100k_base;

/// Counts the approximate number of tokens in a string using cl100k_base.
/// Falls back to a character-based estimation (len / 4) if the tokenizer fails.
pub fn count_tokens(text: &str) -> usize {
    if let Ok(bpe) = cl100k_base() {
        bpe.encode_with_special_tokens(text).len()
    } else {
        // Fallback: ~4 chars per token
        text.len() / 4
    }
}
