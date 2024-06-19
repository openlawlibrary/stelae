//! Utility functions for `md5` hash computation.
use md5::{Digest, Md5};

/// Compute the `md5` hash of a string.
///
/// The result is a hexadecimal string of 32 characters.
#[must_use]
pub fn compute(data: String) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{result:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute() {
        let data = "hello world".to_string();
        let result = compute(data);
        assert_eq!(result, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }

    #[test]
    fn test_compute_empty() {
        let data = "".to_string();
        let result = compute(data);
        assert_eq!(result, "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn test_compute_unicode() {
        let data = "😋".to_string();
        let result = compute(data);
        assert_eq!(result, "a0a836f06f8bd1b45d2f70db1e334b5d");
    }
}
