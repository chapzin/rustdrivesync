use crate::error::{Result, RustDriveSyncError};
use crate::google_drive::auth::DriveAuthenticator;
use crate::google_drive::models::{FolderInfo, ListFilesFilter};
use google_drive3::{
    api::File,
    hyper::{self, client::HttpConnector},
    hyper_rustls, DriveHub,
};
use std::io::Cursor;
use tracing::{debug, info};

/// MIME type para pastas do Google Drive
const FOLDER_MIME_TYPE: &str = "application/vnd.google-apps.folder";

/// Cliente para interagir com Google Drive API
pub struct DriveClient {
    hub: DriveHub<hyper_rustls::HttpsConnector<HttpConnector>>,
    auth: DriveAuthenticator,
}

impl DriveClient {
    /// Cria um novo cliente Drive a partir de um autenticador
    pub async fn new(auth: DriveAuthenticator) -> Result<Self> {
        // Criar cliente HTTP usando os tipos do google-drive3
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(|e| RustDriveSyncError::DriveApiError {
                message: format!("Erro ao configurar HTTPS connector: {}", e),
            })?
            .https_only()
            .enable_http1()
            .build();

        let client = hyper::Client::builder().build(https);

        info!("Cliente Google Drive inicializado com sucesso");

        // Criar hub clonando o authenticator para manter uma referência
        let authenticator = auth.authenticator.clone();

        Ok(Self {
            hub: DriveHub::new(client, authenticator),
            auth,
        })
    }

    /// Cria uma pasta no Google Drive
    ///
    /// # Arguments
    /// * `name` - Nome da pasta
    /// * `parent_id` - ID da pasta pai (None para criar na raiz)
    pub async fn create_folder(&self, name: &str, parent_id: Option<String>) -> Result<FolderInfo> {
        info!("Criando pasta '{}' no Google Drive", name);

        let mut file = File::default();
        file.name = Some(name.to_string());
        file.mime_type = Some(FOLDER_MIME_TYPE.to_string());

        if let Some(parent) = parent_id.as_ref() {
            file.parents = Some(vec![parent.clone()]);
        }

        // Para criar pastas (sem conteúdo), usamos upload com Cursor vazio
        let empty_content = Cursor::new(Vec::new());

        // Parse do MIME type com tratamento de erro adequado
        let mime_type = FOLDER_MIME_TYPE.parse().map_err(|_| {
            RustDriveSyncError::DriveApiError {
                message: format!("MIME type inválido: {}", FOLDER_MIME_TYPE),
            }
        })?;

        let (_response, created_file) = self
            .hub
            .files()
            .create(file)
            .param("fields", "id,name,parents")
            .upload(empty_content, mime_type)
            .await
            .map_err(|e| RustDriveSyncError::DriveApiError {
                message: format!("Erro ao criar pasta: {}", e),
            })?;

        Ok(FolderInfo {
            id: created_file.id.unwrap_or_default(),
            name: created_file.name.unwrap_or_else(|| name.to_string()),
            parent_id,
        })
    }

    /// Busca uma pasta pelo nome
    ///
    /// # Arguments
    /// * `name` - Nome da pasta a buscar
    /// * `parent_id` - ID da pasta pai onde buscar (None para buscar na raiz)
    pub async fn find_folder(&self, name: &str, parent_id: Option<String>) -> Result<Option<FolderInfo>> {
        debug!("Buscando pasta '{}' no Google Drive", name);

        let mut query = format!(
            "mimeType='application/vnd.google-apps.folder' and name='{}' and trashed=false",
            name
        );

        if let Some(parent) = parent_id.as_ref() {
            query.push_str(&format!(" and '{}' in parents", parent));
        }

        let (_response, file_list) = self
            .hub
            .files()
            .list()
            .q(&query)
            .param("fields", "files(id,name,parents)")
            .doit()
            .await
            .map_err(|e| RustDriveSyncError::DriveApiError {
                message: format!("Erro ao buscar pasta: {}", e),
            })?;

        if let Some(files) = file_list.files {
            if let Some(file) = files.first() {
                let parent = file
                    .parents
                    .as_ref()
                    .and_then(|p| p.first())
                    .cloned();

                return Ok(Some(FolderInfo {
                    id: file.id.clone().unwrap_or_default(),
                    name: file.name.clone().unwrap_or_else(|| name.to_string()),
                    parent_id: parent,
                }));
            }
        }

        Ok(None)
    }

    /// Garante que uma pasta existe, criando-a se necessário
    ///
    /// # Arguments
    /// * `name` - Nome da pasta
    /// * `parent_id` - ID da pasta pai
    pub async fn ensure_folder(&self, name: &str, parent_id: Option<String>) -> Result<FolderInfo> {
        // Primeiro tentar encontrar
        if let Some(folder) = self.find_folder(name, parent_id.clone()).await? {
            debug!("Pasta '{}' já existe com ID: {}", name, folder.id);
            return Ok(folder);
        }

        // Se não existe, criar
        self.create_folder(name, parent_id).await
    }

    /// Lista arquivos no Google Drive
    ///
    /// # Arguments
    /// * `filter` - Filtros para a listagem
    pub async fn list_files(&self, filter: ListFilesFilter) -> Result<Vec<File>> {
        debug!("Listando arquivos no Google Drive");

        let mut query_parts = vec!["trashed=false".to_string()];

        if let Some(parent_id) = filter.parent_folder_id {
            query_parts.push(format!("'{}' in parents", parent_id));
        }

        if let Some(name) = filter.name_contains {
            query_parts.push(format!("name contains '{}'", name));
        }

        if let Some(mime_type) = filter.mime_type {
            query_parts.push(format!("mimeType='{}'", mime_type));
        }

        let query = query_parts.join(" and ");

        let mut request = self.hub.files().list().q(&query);

        if let Some(max) = filter.max_results {
            request = request.page_size(max);
        }

        let (_response, file_list) = request.doit().await.map_err(|e| {
            RustDriveSyncError::DriveApiError {
                message: format!("Erro ao listar arquivos: {}", e),
            }
        })?;

        Ok(file_list.files.unwrap_or_default())
    }

    /// Verifica se um arquivo já existe no Drive (por nome e pasta pai)
    ///
    /// # Arguments
    /// * `name` - Nome do arquivo
    /// * `parent_id` - ID da pasta pai
    pub async fn file_exists(&self, name: &str, parent_id: Option<String>) -> Result<Option<String>> {
        debug!("Verificando se arquivo '{}' existe", name);

        let filter = ListFilesFilter {
            parent_folder_id: parent_id,
            name_contains: Some(name.to_string()),
            mime_type: None,
            max_results: Some(1),
        };

        let files = self.list_files(filter).await?;

        // Buscar match exato do nome
        for file in files {
            if file.name.as_ref().map(|n| n == name).unwrap_or(false) {
                return Ok(file.id);
            }
        }

        Ok(None)
    }

    /// Retorna referência ao hub interno (para operações avançadas)
    pub fn hub(&self) -> &DriveHub<hyper_rustls::HttpsConnector<HttpConnector>> {
        &self.hub
    }

    /// Obtém um token de acesso válido para requisições HTTP diretas
    pub async fn get_token(&self) -> Result<String> {
        self.auth.get_token().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Testes de integração reais requerem credenciais válidas
    // Por enquanto, vamos ter testes unitários básicos

    #[test]
    fn test_list_files_filter_default() {
        let filter = ListFilesFilter::default();
        assert!(filter.parent_folder_id.is_none());
        assert!(filter.name_contains.is_none());
    }
}
