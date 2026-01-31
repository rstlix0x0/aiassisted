//! SHA256 checksum implementation.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::core::infra::Checksum;
use crate::core::types::Result;

/// Checksum calculator using SHA256.
#[derive(Debug, Clone, Default)]
pub struct Sha2Checksum;

impl Sha2Checksum {
    /// Create a new Sha2Checksum instance.
    pub fn new() -> Self {
        Self
    }
}

impl Checksum for Sha2Checksum {
    fn sha256(&self, content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    fn sha256_file(&self, path: &Path) -> Result<String> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let checksum = Sha2Checksum::new();
        let result = checksum.sha256(b"");
        assert_eq!(
            result,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hello() {
        let checksum = Sha2Checksum::new();
        let result = checksum.sha256(b"hello");
        assert_eq!(
            result,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
