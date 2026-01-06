use rustdrivesync::error::RustDriveSyncError;

#[test]
fn test_error_display() {
    let error = RustDriveSyncError::ConfigNotFound {
        path: "/test/path".to_string(),
    };

    let message = format!("{}", error);
    assert!(message.contains("/test/path"));
}

#[test]
fn test_error_types() {
    let _config_error = RustDriveSyncError::ConfigInvalid {
        message: "test".to_string(),
    };

    let _auth_error = RustDriveSyncError::AuthenticationFailed {
        message: "test".to_string(),
    };

    let _network_error = RustDriveSyncError::NetworkError {
        message: "test".to_string(),
    };
}
