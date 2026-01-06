use crate::error::{Result, RustDriveSyncError};
use std::path::Path;

// Re-exportar yup_oauth2 para uso em outros módulos
pub use yup_oauth2;

use yup_oauth2::{
    authenticator::Authenticator, hyper::client::HttpConnector, hyper_rustls::HttpsConnector,
    read_application_secret, InstalledFlowAuthenticator, InstalledFlowReturnMethod,
};

/// Gerenciador de autenticação OAuth2 para Google Drive
pub struct DriveAuthenticator {
    pub(crate) authenticator: Authenticator<HttpsConnector<HttpConnector>>,
    scopes: Vec<String>,
}

impl DriveAuthenticator {
    /// Cria um novo autenticador a partir de arquivos de credenciais
    ///
    /// # Arguments
    /// * `credentials_path` - Caminho para o arquivo credentials.json
    /// * `token_path` - Caminho para salvar/carregar o token.json
    /// * `scopes` - Lista de escopos OAuth2 necessários
    pub async fn new<P: AsRef<Path>>(
        credentials_path: P,
        token_path: P,
        scopes: Vec<String>,
    ) -> Result<Self> {
        let credentials_path_ref = credentials_path.as_ref();
        let token_path_ref = token_path.as_ref();

        // Verificar se o arquivo de credenciais existe
        if !credentials_path_ref.exists() {
            return Err(RustDriveSyncError::CredentialsNotFound {
                path: credentials_path_ref.display().to_string(),
            });
        }

        // Ler o application secret do arquivo de credenciais
        let secret = read_application_secret(credentials_path_ref)
            .await
            .map_err(|e| RustDriveSyncError::AuthenticationFailed {
                message: format!("Erro ao ler credenciais: {}", e),
            })?;

        // Criar o autenticador
        let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
            .persist_tokens_to_disk(token_path_ref)
            .build()
            .await
            .map_err(|e| RustDriveSyncError::AuthenticationFailed {
                message: format!("Erro ao criar autenticador: {}", e),
            })?;

        Ok(Self {
            authenticator: auth,
            scopes,
        })
    }

    /// Obtém um token de acesso válido
    ///
    /// Este método automaticamente renova o token se necessário
    pub async fn get_token(&self) -> Result<String> {
        let token: yup_oauth2::AccessToken = self
            .authenticator
            .token(&self.scopes)
            .await
            .map_err(|e| RustDriveSyncError::AuthenticationFailed {
                message: format!("Erro ao obter token: {}", e),
            })?;

        Ok(token.token().unwrap_or_default().to_string())
    }

    /// Verifica se há um token válido salvo
    pub async fn has_valid_token(&self) -> bool {
        let result: std::result::Result<yup_oauth2::AccessToken, _> =
            self.authenticator.token(&self.scopes).await;
        result.is_ok()
    }

    /// Força uma nova autenticação (útil para renovar tokens expirados)
    pub async fn refresh_token(&self) -> Result<String> {
        self.get_token().await
    }
}

/// Inicia o fluxo de autenticação OAuth2 interativo
///
/// Esta função:
/// 1. Lê as credenciais do arquivo
/// 2. Abre o navegador para autorização
/// 3. Aguarda o callback do Google
/// 4. Salva o token no arquivo especificado
pub async fn authenticate<P: AsRef<Path>>(
    credentials_path: P,
    token_path: P,
    scopes: Vec<String>,
) -> Result<()> {
    tracing::info!("Iniciando fluxo de autenticação OAuth2...");

    let authenticator = DriveAuthenticator::new(credentials_path, token_path, scopes.clone()).await?;

    // Forçar obtenção de token (isso abrirá o navegador se necessário)
    let _token = authenticator.get_token().await?;

    tracing::info!("✓ Autenticação concluída com sucesso!");
    tracing::info!("Token salvo. Você já pode usar o RustDriveSync.");

    Ok(())
}

/// Revoga o token atual (remove o arquivo de token)
pub async fn revoke_token<P: AsRef<Path>>(token_path: P) -> Result<()> {
    let token_path_ref = token_path.as_ref();

    if token_path_ref.exists() {
        std::fs::remove_file(token_path_ref).map_err(|e| {
            RustDriveSyncError::AuthenticationFailed {
                message: format!("Erro ao revogar token: {}", e),
            }
        })?;
        tracing::info!("✓ Token revogado com sucesso");
    } else {
        tracing::warn!("Nenhum token encontrado para revogar");
    }

    Ok(())
}

/// Verifica o status da autenticação
pub async fn check_auth_status<P: AsRef<Path> + Clone>(
    credentials_path: P,
    token_path: P,
    scopes: Vec<String>,
) -> Result<()> {
    let token_path_ref = token_path.as_ref();

    if !token_path_ref.exists() {
        println!("❌ Não autenticado");
        println!("Execute 'rustdrivesync auth' para autenticar");
        return Ok(());
    }

    let token_path_display = token_path_ref.display().to_string();
    let authenticator = DriveAuthenticator::new(credentials_path, token_path, scopes).await?;

    if authenticator.has_valid_token().await {
        println!("✓ Autenticado com sucesso");
        println!("Token válido encontrado em: {}", token_path_display);
    } else {
        println!("⚠ Token expirado ou inválido");
        println!("Execute 'rustdrivesync auth' para renovar");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_missing_credentials() {
        let result = DriveAuthenticator::new(
            "/nonexistent/credentials.json",
            "/tmp/token.json",
            vec!["https://www.googleapis.com/auth/drive.file".to_string()],
        )
        .await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, RustDriveSyncError::CredentialsNotFound { .. }));
        }
    }
}
