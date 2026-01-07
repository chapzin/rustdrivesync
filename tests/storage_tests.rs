// Testes unitários para módulo storage

use rustdrivesync::storage::models::*;

#[test]
fn test_upload_options_with_parent() {
    let opts = UploadOptions::with_parent("folder_123".to_string());
    assert_eq!(opts.parent_folder_id, Some("folder_123".to_string()));
    assert!(opts.use_resumable);
    assert!(opts.verify_checksum);
}

#[test]
fn test_upload_options_resumable() {
    let opts = UploadOptions::resumable();
    assert!(opts.use_resumable);
    assert!(opts.verify_checksum);
    assert!(opts.parent_folder_id.is_none());
}

#[test]
fn test_upload_options_default() {
    let opts = UploadOptions::default();
    assert!(!opts.use_resumable);
    assert!(!opts.verify_checksum);
    assert!(opts.parent_folder_id.is_none());
}

#[test]
fn test_storage_stats_new() {
    let stats = StorageStats::new(500_000_000, Some(1_000_000_000));
    assert_eq!(stats.used_bytes, 500_000_000);
    assert_eq!(stats.limit_bytes, Some(1_000_000_000));
    assert_eq!(stats.usage_percentage, 50.0);
}

#[test]
fn test_storage_stats_unlimited() {
    let stats = StorageStats::new(500_000_000, None);
    assert_eq!(stats.used_bytes, 500_000_000);
    assert!(stats.limit_bytes.is_none());
    assert_eq!(stats.usage_percentage, 0.0);
}

#[test]
fn test_storage_stats_is_near_limit() {
    let stats_normal = StorageStats::new(500_000_000, Some(1_000_000_000)); // 50%
    assert!(!stats_normal.is_near_limit());

    let stats_near = StorageStats::new(950_000_000, Some(1_000_000_000)); // 95%
    assert!(stats_near.is_near_limit());
}

#[test]
fn test_storage_stats_is_over_limit() {
    let stats_normal = StorageStats::new(500_000_000, Some(1_000_000_000)); // 50%
    assert!(!stats_normal.is_over_limit());

    let stats_at_limit = StorageStats::new(1_000_000_000, Some(1_000_000_000)); // 100%
    assert!(stats_at_limit.is_over_limit());

    let stats_over = StorageStats::new(1_100_000_000, Some(1_000_000_000)); // 110%
    assert!(stats_over.is_over_limit());
}

#[test]
fn test_storage_stats_zero_limit() {
    let stats = StorageStats::new(100, Some(0));
    assert_eq!(stats.usage_percentage, 0.0);
}

#[test]
fn test_upload_result_defaults() {
    let result = UploadResult {
        file_id: "file_123".to_string(),
        name: "test.txt".to_string(),
        size: 1024,
        md5_checksum: Some("abc123".to_string()),
        web_view_link: Some("https://drive.google.com/file/...".to_string()),
        upload_duration_secs: 1.5,
    };

    assert_eq!(result.file_id, "file_123");
    assert_eq!(result.name, "test.txt");
    assert_eq!(result.size, 1024);
    assert!(result.md5_checksum.is_some());
    assert!(result.web_view_link.is_some());
    assert_eq!(result.upload_duration_secs, 1.5);
}

#[test]
fn test_folder_info() {
    let folder = FolderInfo {
        id: "folder_123".to_string(),
        name: "My Folder".to_string(),
        parent_id: Some("parent_456".to_string()),
    };

    assert_eq!(folder.id, "folder_123");
    assert_eq!(folder.name, "My Folder");
    assert_eq!(folder.parent_id, Some("parent_456".to_string()));
}

#[test]
fn test_file_info() {
    let file = FileInfo {
        id: "file_123".to_string(),
        name: "document.pdf".to_string(),
        size: 1048576, // 1MB
        mime_type: "application/pdf".to_string(),
        modified_time: Some(1640000000),
        md5_checksum: Some("abc123def456".to_string()),
    };

    assert_eq!(file.id, "file_123");
    assert_eq!(file.name, "document.pdf");
    assert_eq!(file.size, 1048576);
    assert_eq!(file.mime_type, "application/pdf");
    assert!(file.modified_time.is_some());
    assert!(file.md5_checksum.is_some());
}