use thiserror::Error;

/// Tipos de erro do RustDriveSync
#[derive(Error, Debug)]
pub enum RustDriveSyncError {
    // Erros de Configuração
    #[error("Arquivo de configuração não encontrado: {path}")]
    ConfigNotFound { path: String },

    #[error("Configuração inválida: {message}")]
    ConfigInvalid { message: String },

    // Erros de Autenticação
    #[error("Falha na autenticação: {message}")]
    AuthenticationFailed { message: String },

    #[error("Token expirado e não foi possível renovar")]
    TokenExpired,

    #[error("Credenciais não encontradas: {path}")]
    CredentialsNotFound { path: String },

    // Erros de Rede
    #[error("Erro de conexão: {message}")]
    NetworkError { message: String },

    #[error("Rate limit excedido. Aguarde {retry_after} segundos")]
    RateLimitExceeded { retry_after: u64 },

    #[error("Timeout na requisição")]
    RequestTimeout,

    // Erros de Arquivo
    #[error("Arquivo não encontrado: {path}")]
    FileNotFound { path: String },

    #[error("Erro ao ler arquivo {path}: {message}")]
    FileReadError { path: String, message: String },

    #[error("Permissão negada: {path}")]
    PermissionDenied { path: String },

    #[error("Arquivo muito grande: {size} bytes (máximo: {max_size} bytes)")]
    FileTooLarge { size: u64, max_size: u64 },

    // Erros do Google Drive
    #[error("Pasta de destino não encontrada no Drive: {folder_id}")]
    DriveFolderNotFound { folder_id: String },

    #[error("Cota do Google Drive excedida")]
    DriveQuotaExceeded,

    #[error("Erro da API do Google Drive: {message}")]
    DriveApiError { message: String },

    // Erros de Upload
    #[error("Upload falhou após {attempts} tentativas: {message}")]
    UploadFailed { attempts: u32, message: String },

    #[error("Checksum MD5 não corresponde para {file}: esperado {expected}, obtido {actual}")]
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },

    #[error("Verificação de integridade falhou para: {path}")]
    IntegrityCheckFailed { path: String },

    // Erros de Estado
    #[error("Erro no gerenciamento de estado: {message}")]
    StateError { message: String },

    // Erros de I/O
    #[error("Erro de I/O: {0}")]
    IoError(#[from] std::io::Error),

    // Erros genéricos
    #[error("Erro: {0}")]
    Other(#[from] anyhow::Error),
}
