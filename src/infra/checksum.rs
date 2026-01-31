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
    use std::io::Write;
    use tempfile::NamedTempFile;

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

    #[test]
    fn test_sha256_unicode() {
        let checksum = Sha2Checksum::new();
        let result = checksum.sha256("Hello, 世界! 🦀".as_bytes());
        // Actual SHA256 hash for this UTF-8 string
        assert_eq!(
            result,
            "f7beeef5a5fd2c53200bde7be4eebac036d0a1c4adac8afc5a53a92ee7a9d767"
        );
    }

    #[test]
    fn test_sha256_large_data() {
        let checksum = Sha2Checksum::new();
        let large_data = vec![0u8; 1_000_000]; // 1MB of zeros
        let result = checksum.sha256(&large_data);
        // This should not panic and should produce a valid hash
        assert_eq!(result.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_sha256_file_success() {
        let checksum = Sha2Checksum::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        temp_file.flush().unwrap();

        let result = checksum.sha256_file(temp_file.path()).unwrap();
        // SHA256 of "test content"
        assert_eq!(
            result,
            "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72"
        );
    }

    #[test]
    fn test_sha256_file_large() {
        let checksum = Sha2Checksum::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        // Write 100KB of data (larger than buffer size)
        let data = vec![b'x'; 100_000];
        temp_file.write_all(&data).unwrap();
        temp_file.flush().unwrap();

        let result = checksum.sha256_file(temp_file.path()).unwrap();
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_sha256_file_not_found() {
        let checksum = Sha2Checksum::new();
        let result = checksum.sha256_file(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_sha256_file_empty() {
        let checksum = Sha2Checksum::new();
        let temp_file = NamedTempFile::new().unwrap();

        let result = checksum.sha256_file(temp_file.path()).unwrap();
        // SHA256 of empty file
        assert_eq!(
            result,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_consistency() {
        let checksum = Sha2Checksum::new();
        let data = b"consistency test";

        // Hash the same data multiple times
        let hash1 = checksum.sha256(data);
        let hash2 = checksum.sha256(data);
        let hash3 = checksum.sha256(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }
}
