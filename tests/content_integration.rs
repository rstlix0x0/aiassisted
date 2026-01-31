//! Integration tests for content module.
//!
//! These tests verify that manifest loading, GitHub downloading, and sync
//! operations work correctly together using real implementations.

use aiassisted::core::infra::{Checksum, FileSystem, HttpClient};
use aiassisted::core::types::ManifestEntry;
use aiassisted::infra::{ReqwestClient, Sha2Checksum, StdFileSystem};
use aiassisted::Manifest;
use std::path::PathBuf;
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};


#[tokio::test]
async fn test_manifest_load_save_roundtrip() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");

    // Create a manifest
    let original = Manifest {
        version: "1.0.0".to_string(),
        files: vec![
            ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: "abc123".to_string(),
            },
            ManifestEntry {
                path: PathBuf::from("dir/file2.txt"),
                checksum: "def456".to_string(),
            },
        ],
    };

    // Save it
    original.save(&fs, &manifest_path).await.unwrap();

    // Load it back
    let loaded = Manifest::load_local(&fs, &manifest_path).await.unwrap();

    // Verify
    assert_eq!(loaded.version, original.version);
    assert_eq!(loaded.files.len(), original.files.len());
    assert_eq!(loaded.files[0].path, original.files[0].path);
    assert_eq!(loaded.files[0].checksum, original.files[0].checksum);
}

#[tokio::test]
async fn test_manifest_diff_workflow() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();

    // Local manifest (v1.0.0)
    let local = Manifest {
        version: "1.0.0".to_string(),
        files: vec![
            ManifestEntry {
                path: PathBuf::from("unchanged.txt"),
                checksum: "same".to_string(),
            },
            ManifestEntry {
                path: PathBuf::from("modified.txt"),
                checksum: "old_hash".to_string(),
            },
        ],
    };

    // Remote manifest (v2.0.0)
    let remote = Manifest {
        version: "2.0.0".to_string(),
        files: vec![
            ManifestEntry {
                path: PathBuf::from("unchanged.txt"),
                checksum: "same".to_string(),
            },
            ManifestEntry {
                path: PathBuf::from("modified.txt"),
                checksum: "new_hash".to_string(),
            },
            ManifestEntry {
                path: PathBuf::from("new_file.txt"),
                checksum: "new".to_string(),
            },
        ],
    };

    // Save local manifest
    let local_path = temp_dir.path().join("local_manifest.json");
    local.save(&fs, &local_path).await.unwrap();

    // Calculate diff
    let diff = local.diff(&remote);

    // Verify diff
    assert!(diff.has_changes());
    assert_eq!(diff.unchanged_files.len(), 1);
    assert_eq!(diff.modified_files.len(), 1);
    assert_eq!(diff.new_files.len(), 1);

    // Verify files to download
    let to_download = diff.files_to_download();
    assert_eq!(to_download.len(), 2); // 1 modified + 1 new
}

#[tokio::test]
async fn test_checksum_verification_workflow() {
    let fs = StdFileSystem::new();
    let checksum = Sha2Checksum::new();
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    let file1_path = temp_dir.path().join("file1.txt");
    let file2_path = temp_dir.path().join("file2.txt");

    fs.write(&file1_path, "content1").await.unwrap();
    fs.write(&file2_path, "content2").await.unwrap();

    // Calculate checksums
    let hash1 = checksum.sha256(b"content1");
    let hash2 = checksum.sha256(b"content2");

    // Create manifest with checksums
    let manifest = Manifest {
        version: "1.0.0".to_string(),
        files: vec![
            ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: hash1.clone(),
            },
            ManifestEntry {
                path: PathBuf::from("file2.txt"),
                checksum: hash2.clone(),
            },
        ],
    };

    // Verify checksums
    let results = manifest
        .verify_checksums(&checksum, &fs, temp_dir.path())
        .unwrap();

    // All should match
    assert_eq!(results.len(), 2);
    assert!(results[0].1); // file1 matches
    assert!(results[1].1); // file2 matches
}

#[tokio::test]
async fn test_checksum_verification_with_mismatch() {
    let fs = StdFileSystem::new();
    let checksum = Sha2Checksum::new();
    let temp_dir = TempDir::new().unwrap();

    // Create test file
    let file_path = temp_dir.path().join("file.txt");
    fs.write(&file_path, "actual content").await.unwrap();

    // Create manifest with wrong checksum
    let manifest = Manifest {
        version: "1.0.0".to_string(),
        files: vec![ManifestEntry {
            path: PathBuf::from("file.txt"),
            checksum: "wrong_checksum".to_string(),
        }],
    };

    // Verify checksums
    let results = manifest
        .verify_checksums(&checksum, &fs, temp_dir.path())
        .unwrap();

    // Should not match
    assert_eq!(results.len(), 1);
    assert!(!results[0].1); // Mismatch
}

#[tokio::test]
async fn test_download_and_verify_workflow() {
    // Setup mock HTTP server
    let mock_server = MockServer::start().await;

    let file_content = "Hello, World!";
    let checksum = Sha2Checksum::new();
    let expected_hash = checksum.sha256(file_content.as_bytes());

    // Mock the HTTP endpoint
    Mock::given(method("GET"))
        .and(path("/test.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string(file_content))
        .mount(&mock_server)
        .await;

    // Setup filesystem
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("test.txt");

    // Download file
    let http = ReqwestClient::new();
    let url = format!("{}/test.txt", mock_server.uri());
    http.download(&url, &dest_path).await.unwrap();

    // Verify file was downloaded
    assert!(fs.exists(&dest_path));

    // Verify content
    let content = fs.read(&dest_path).await.unwrap();
    assert_eq!(content, file_content);

    // Verify checksum
    let actual_hash = checksum.sha256_file(&dest_path).unwrap();
    assert_eq!(actual_hash, expected_hash);
}

#[tokio::test]
async fn test_manifest_download_and_parse() {
    // Setup mock HTTP server
    let mock_server = MockServer::start().await;

    let manifest_json = r#"{
        "version": "1.0.0",
        "files": [
            {"path": "file1.txt", "checksum": "abc123"},
            {"path": "file2.txt", "checksum": "def456"}
        ]
    }"#;

    Mock::given(method("GET"))
        .and(path("/manifest.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(manifest_json))
        .mount(&mock_server)
        .await;

    // Download and parse manifest
    let http = ReqwestClient::new();
    let url = format!("{}/manifest.json", mock_server.uri());

    let manifest = Manifest::load_remote(&http, &url).await.unwrap();

    // Verify
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.files.len(), 2);
    assert_eq!(manifest.files[0].path, PathBuf::from("file1.txt"));
    assert_eq!(manifest.files[0].checksum, "abc123");
}

#[tokio::test]
async fn test_full_download_workflow_with_checksum_verification() {
    // Setup mock HTTP server
    let mock_server = MockServer::start().await;

    let file1_content = "File 1 content";
    let file2_content = "File 2 content";

    let checksum = Sha2Checksum::new();
    let hash1 = checksum.sha256(file1_content.as_bytes());
    let hash2 = checksum.sha256(file2_content.as_bytes());

    // Create manifest
    let manifest = Manifest {
        version: "1.0.0".to_string(),
        files: vec![
            ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: hash1.clone(),
            },
            ManifestEntry {
                path: PathBuf::from("subdir/file2.txt"),
                checksum: hash2.clone(),
            },
        ],
    };

    // Mock file downloads
    Mock::given(method("GET"))
        .and(path("/file1.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string(file1_content))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/subdir/file2.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string(file2_content))
        .mount(&mock_server)
        .await;

    // Setup filesystem
    let fs = StdFileSystem::new();
    let http = ReqwestClient::new();
    let temp_dir = TempDir::new().unwrap();

    // Download all files
    for entry in &manifest.files {
        let url = format!("{}/{}", mock_server.uri(), entry.path.display());
        let dest = temp_dir.path().join(&entry.path);

        // Create parent directory if needed
        if let Some(parent) = dest.parent() {
            fs.create_dir_all(parent).await.unwrap();
        }

        // Download
        http.download(&url, &dest).await.unwrap();

        // Verify checksum
        let actual_hash = checksum.sha256_file(&dest).unwrap();
        assert_eq!(
            actual_hash, entry.checksum,
            "Checksum mismatch for {}",
            entry.path.display()
        );
    }

    // Verify all files exist
    assert!(fs.exists(&temp_dir.path().join("file1.txt")));
    assert!(fs.exists(&temp_dir.path().join("subdir/file2.txt")));

    // Verify using manifest's verify_checksums
    let results = manifest
        .verify_checksums(&checksum, &fs, temp_dir.path())
        .unwrap();

    assert_eq!(results.len(), 2);
    assert!(results[0].1); // file1 matches
    assert!(results[1].1); // file2 matches
}

#[tokio::test]
async fn test_update_workflow_with_partial_changes() {
    let fs = StdFileSystem::new();
    let checksum = Sha2Checksum::new();
    let temp_dir = TempDir::new().unwrap();

    // Setup initial state (v1.0.0)
    let content1 = "Original content 1";
    let content2 = "Original content 2";

    let file1_path = temp_dir.path().join("file1.txt");
    let file2_path = temp_dir.path().join("file2.txt");

    fs.write(&file1_path, content1).await.unwrap();
    fs.write(&file2_path, content2).await.unwrap();

    let hash1_old = checksum.sha256(content1.as_bytes());
    let hash2_old = checksum.sha256(content2.as_bytes());

    let old_manifest = Manifest {
        version: "1.0.0".to_string(),
        files: vec![
            ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: hash1_old.clone(),
            },
            ManifestEntry {
                path: PathBuf::from("file2.txt"),
                checksum: hash2_old.clone(),
            },
        ],
    };

    // New version (v2.0.0) - only file1 changed
    let content1_new = "Updated content 1";
    let hash1_new = checksum.sha256(content1_new.as_bytes());

    let new_manifest = Manifest {
        version: "2.0.0".to_string(),
        files: vec![
            ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: hash1_new.clone(),
            },
            ManifestEntry {
                path: PathBuf::from("file2.txt"),
                checksum: hash2_old.clone(), // Unchanged
            },
        ],
    };

    // Calculate diff
    let diff = old_manifest.diff(&new_manifest);

    // Should have 1 modified, 1 unchanged
    assert_eq!(diff.modified_files.len(), 1);
    assert_eq!(diff.unchanged_files.len(), 1);
    assert_eq!(diff.new_files.len(), 0);

    // Files to download should only include the modified one
    let to_download = diff.files_to_download();
    assert_eq!(to_download.len(), 1);
    assert_eq!(to_download[0].path, PathBuf::from("file1.txt"));
}

#[tokio::test]
async fn test_network_error_handling() {
    let http = ReqwestClient::new();

    // Try to download from invalid URL
    let result = http.get("http://localhost:1/nonexistent").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_large_file_download_and_checksum() {
    let mock_server = MockServer::start().await;

    // Create large content (10KB - smaller for faster test)
    let large_content = "x".repeat(10_000);
    let checksum = Sha2Checksum::new();
    let expected_hash = checksum.sha256(large_content.as_bytes());

    Mock::given(method("GET"))
        .and(path("/large.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(large_content.as_bytes()))
        .mount(&mock_server)
        .await;

    let fs = StdFileSystem::new();
    let http = ReqwestClient::new();
    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("large.txt");

    // Download
    let url = format!("{}/large.txt", mock_server.uri());
    http.download(&url, &dest_path).await.unwrap();

    // Verify file exists and has content
    assert!(fs.exists(&dest_path));
    let content = fs.read(&dest_path).await.unwrap();
    assert_eq!(content.len(), large_content.len());

    // Verify checksum
    let actual_hash = checksum.sha256_file(&dest_path).unwrap();
    assert_eq!(actual_hash, expected_hash);
}
