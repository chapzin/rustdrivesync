use rustdrivesync::error::RustDriveSyncError;

// Testes de erros de configuração
#[test]
fn test_config_not_found_error() {
    let error = RustDriveSyncError::ConfigNotFound {
        path: "/test/config.toml".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("não encontrado"));
    assert!(message.contains("/test/config.toml"));
}

#[test]
fn test_config_invalid_error() {
    let error = RustDriveSyncError::ConfigInvalid {
        message: "campo obrigatório ausente".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("inválida"));
    assert!(message.contains("campo obrigatório ausente"));
}

// Testes de erros de autenticação
#[test]
fn test_authentication_failed_error() {
    let error = RustDriveSyncError::AuthenticationFailed {
        message: "credenciais inválidas".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("autenticação"));
    assert!(message.contains("credenciais inválidas"));
}

#[test]
fn test_token_expired_error() {
    let error = RustDriveSyncError::TokenExpired;
    let message = format!("{}", error);
    assert!(message.contains("expirado"));
}

#[test]
fn test_credentials_not_found_error() {
    let error = RustDriveSyncError::CredentialsNotFound {
        path: "/home/user/.credentials.json".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("Credenciais não encontradas"));
    assert!(message.contains(".credentials.json"));
}

// Testes de erros de rede
#[test]
fn test_network_error() {
    let error = RustDriveSyncError::NetworkError {
        message: "conexão recusada".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("conexão"));
    assert!(message.contains("conexão recusada"));
}

#[test]
fn test_rate_limit_exceeded_error() {
    let error = RustDriveSyncError::RateLimitExceeded { retry_after: 300 };
    let message = format!("{}", error);
    assert!(message.contains("Rate limit"));
    assert!(message.contains("300"));
}

#[test]
fn test_request_timeout_error() {
    let error = RustDriveSyncError::RequestTimeout;
    let message = format!("{}", error);
    assert!(message.contains("Timeout"));
}

// Testes de erros de arquivo
#[test]
fn test_file_not_found_error() {
    let error = RustDriveSyncError::FileNotFound {
        path: "/path/to/missing.txt".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("não encontrado"));
    assert!(message.contains("missing.txt"));
}

#[test]
fn test_file_read_error() {
    let error = RustDriveSyncError::FileReadError {
        path: "/test/file.txt".to_string(),
        message: "permissão negada".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("ler arquivo"));
    assert!(message.contains("/test/file.txt"));
    assert!(message.contains("permissão negada"));
}

#[test]
fn test_permission_denied_error() {
    let error = RustDriveSyncError::PermissionDenied {
        path: "/root/secret.txt".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("Permissão negada"));
    assert!(message.contains("/root/secret.txt"));
}

#[test]
fn test_file_too_large_error() {
    let error = RustDriveSyncError::FileTooLarge {
        size: 524_288_000,
        max_size: 104_857_600,
    };
    let message = format!("{}", error);
    assert!(message.contains("muito grande"));
    assert!(message.contains("524288000"));
    assert!(message.contains("104857600"));
}

// Testes de erros do Google Drive
#[test]
fn test_drive_folder_not_found_error() {
    let error = RustDriveSyncError::DriveFolderNotFound {
        folder_id: "1234567890".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("não encontrada"));
    assert!(message.contains("1234567890"));
}

#[test]
fn test_drive_quota_exceeded_error() {
    let error = RustDriveSyncError::DriveQuotaExceeded;
    let message = format!("{}", error);
    assert!(message.contains("Cota"));
    assert!(message.contains("excedida"));
}

#[test]
fn test_drive_api_error() {
    let error = RustDriveSyncError::DriveApiError {
        message: "401 Unauthorized".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("API"));
    assert!(message.contains("401 Unauthorized"));
}

// Testes de erros de upload
#[test]
fn test_upload_failed_error() {
    let error = RustDriveSyncError::UploadFailed {
        attempts: 3,
        message: "timeout na conexão".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("falhou"));
    assert!(message.contains("3"));
    assert!(message.contains("timeout na conexão"));
}

#[test]
fn test_checksum_mismatch_error() {
    let error = RustDriveSyncError::ChecksumMismatch {
        file: "document.pdf".to_string(),
        expected: "abc123".to_string(),
        actual: "def456".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("não corresponde"));
    assert!(message.contains("document.pdf"));
    assert!(message.contains("abc123"));
    assert!(message.contains("def456"));
}

#[test]
fn test_integrity_check_failed_error() {
    let error = RustDriveSyncError::IntegrityCheckFailed {
        path: "/data/corrupted.bin".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("integridade"));
    assert!(message.contains("corrupted.bin"));
}

// Testes de erros de estado
#[test]
fn test_state_error() {
    let error = RustDriveSyncError::StateError {
        message: "falha ao salvar estado".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("estado"));
    assert!(message.contains("falha ao salvar estado"));
}

// Teste de conversão de std::io::Error
#[test]
fn test_io_error_conversion() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "arquivo não encontrado");
    let error: RustDriveSyncError = io_error.into();
    let message = format!("{}", error);
    assert!(message.contains("I/O"));
}
