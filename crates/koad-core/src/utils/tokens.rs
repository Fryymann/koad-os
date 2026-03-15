//! Token Counting Utilities

use tiktoken_rs::cl100k_base;

/// Estimates the token count of a string using the cl100k_base encoder (Gemma/GPT-4 compatible).
pub fn count_tokens(text: &str) -> usize {
    let bpe = cl100k_base().unwrap();
    bpe.encode_with_special_tokens(text).len()
}
