use crate::error::{Result, RustDriveSyncError};
use crate::google_drive::client::DriveClient;
use crate::google_drive::models::{LocalFileMetadata, UploadOptions, UploadResult};
use google_drive3::api::File as DriveFile;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;
use tracing::{debug, info, warn};

/// Gerenciador de uploads para Google Drive
pub struct DriveUploader<'a> {
    client: &'a DriveClient,
}

impl<'a> DriveUploader<'a> {
    /// Cria um novo uploader
    pub fn new(client: &'a DriveClient) -> Self {
        Self { client }
    }

    /// Faz upload de um arquivo para o Google Drive
    ///
    /// # Arguments
    /// * `file_path` - Caminho do arquivo local
    /// * `options` - Opções de upload
    pub async fn upload_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        options: UploadOptions,
    ) -> Result<UploadResult> {
        let file_path = file_path.as_ref();

        // Preparar metadados do arquivo
        let metadata = self.prepare_file_metadata(file_path)?;

        info!(
            "Iniciando upload: {} ({} bytes)",
            metadata.name, metadata.size
        );

        // Decidir entre upload simples ou resumível
        let start_time = Instant::now();

        let result = if options.use_resumable && metadata.size > (5 * 1024 * 1024) {
            debug!("Usando upload resumível (arquivo grande)");
            self.upload_resumable(&metadata, options).await?
        } else {
            debug!("Usando upload simples");
            self.upload_simple(&metadata, options).await?
        };

        let duration = start_time.elapsed().as_secs_f64();

        info!(
            "Upload concluído: {} em {:.2}s ({:.2} MB/s)",
            result.name,
            duration,
            (result.size as f64 / 1_048_576.0) / duration
        );

        Ok(UploadResult {
            upload_duration_secs: duration,
            ..result
        })
    }

    /// Upload simples para arquivos pequenos
    async fn upload_simple(
        &self,
        metadata: &LocalFileMetadata,
        options: UploadOptions,
    ) -> Result<UploadResult> {
        // Ler arquivo completo em memória
        let file_content = fs::read(&metadata.path).map_err(|e| {
            RustDriveSyncError::FileReadError {
                path: metadata.path.display().to_string(),
                message: e.to_string(),
            }
        })?;

        // Criar objeto File para o Drive
        let mut drive_file = DriveFile::default();
        drive_file.name = Some(metadata.name.clone());
        drive_file.mime_type = Some(metadata.mime_type.clone());

        if let Some(parent_id) = options.parent_folder_id.as_ref() {
            drive_file.parents = Some(vec![parent_id.clone()]);
        }

        // Criar um Cursor que implementa Read + Seek
        let cursor = Cursor::new(file_content);

        // Executar upload
        let result = self
            .client
            .hub()
            .files()
            .create(drive_file)
            .upload(
                cursor,
                metadata.mime_type.parse().map_err(|_| {
                    RustDriveSyncError::DriveApiError {
                        message: format!("MIME type inválido: {}", metadata.mime_type),
                    }
                })?,
            )
            .await
            .map_err(|e| RustDriveSyncError::UploadFailed {
                attempts: 1,
                message: format!("Erro no upload simples de {}: {}", metadata.name, e),
            })?;

        let uploaded_file = result.1;

        // Verificar checksum se solicitado
        if options.verify_checksum {
            self.verify_checksum(&uploaded_file, metadata)?;
        }

        Ok(UploadResult {
            file_id: uploaded_file.id.unwrap_or_default(),
            name: uploaded_file.name.unwrap_or_else(|| metadata.name.clone()),
            size: metadata.size,
            md5_checksum: uploaded_file.md5_checksum.clone(),
            web_view_link: uploaded_file.web_view_link.clone(),
            upload_duration_secs: 0.0, // Será preenchido pelo caller
        })
    }

    /// Upload resumível com chunks para arquivos grandes
    /// Agora usa streaming para memória constante
    async fn upload_resumable(
        &self,
        metadata: &LocalFileMetadata,
        options: UploadOptions,
    ) -> Result<UploadResult> {
        // Para arquivos grandes, usar upload com streaming
        // Isso mantém uso de memória constante (~5MB) independente do tamanho do arquivo
        self.upload_streaming(metadata, options).await
    }

    /// Prepara os metadados de um arquivo local
    fn prepare_file_metadata(&self, file_path: &Path) -> Result<LocalFileMetadata> {
        if !file_path.exists() {
            return Err(RustDriveSyncError::FileNotFound {
                path: file_path.display().to_string(),
            });
        }

        let metadata_fs = fs::metadata(file_path).map_err(|e| {
            RustDriveSyncError::FileReadError {
                path: file_path.display().to_string(),
                message: e.to_string(),
            }
        })?;

        let name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| RustDriveSyncError::DriveApiError {
                message: format!("Nome de arquivo inválido: {:?}", file_path),
            })?
            .to_string();

        let mime_type = self.detect_mime_type(file_path);

        // Para arquivos pequenos (< 5MB), calcular MD5 agora
        // Para arquivos grandes, MD5 será calculado durante streaming
        let md5_hash = if metadata_fs.len() < 5 * 1024 * 1024 {
            Some(self.calculate_md5(file_path)?)
        } else {
            None // Será calculado em upload_streaming
        };

        Ok(LocalFileMetadata {
            path: file_path.to_path_buf(),
            name,
            size: metadata_fs.len(),
            mime_type,
            md5_hash,
        })
    }

    /// Detecta o tipo MIME de um arquivo usando mime_guess
    fn detect_mime_type(&self, file_path: &Path) -> String {
        mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string()
    }

    /// Calcula o hash MD5 de um arquivo (síncrono, para arquivos pequenos)
    fn calculate_md5(&self, file_path: &Path) -> Result<String> {
        use md5::{Digest, Md5};

        let content = fs::read(file_path).map_err(|e| RustDriveSyncError::FileReadError {
            path: file_path.display().to_string(),
            message: e.to_string(),
        })?;

        let mut hasher = Md5::new();
        hasher.update(&content);
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    /// Calcula o hash MD5 de um arquivo usando streaming (async, para arquivos grandes)
    async fn calculate_md5_streaming(&self, file_path: &Path) -> Result<String> {
        use md5::{Digest, Md5};

        let mut file = AsyncFile::open(file_path).await.map_err(|e| {
            RustDriveSyncError::FileReadError {
                path: file_path.display().to_string(),
                message: e.to_string(),
            }
        })?;

        let mut hasher = Md5::new();
        let mut buffer = vec![0u8; 256 * 1024]; // 256KB chunks

        loop {
            let n = file.read(&mut buffer).await.map_err(|e| {
                RustDriveSyncError::FileReadError {
                    path: file_path.display().to_string(),
                    message: e.to_string(),
                }
            })?;

            if n == 0 {
                break;
            }

            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Upload com streaming para arquivos grandes usando reqwest diretamente
    async fn upload_streaming(
        &self,
        metadata: &LocalFileMetadata,
        options: UploadOptions,
    ) -> Result<UploadResult> {

        info!("Usando upload streaming para: {}", metadata.name);

        // 1. Calcular MD5 antes do upload (necessário para verificação)
        debug!("Calculando MD5 do arquivo...");
        let md5_hash = self.calculate_md5_streaming(&metadata.path).await?;
        debug!("MD5 calculado: {}", md5_hash);

        // 2. Obter token de acesso
        let token = self.client.get_token().await?;

        // 3. Criar metadados do arquivo para o Google Drive
        let mut file_metadata = serde_json::json!({
            "name": metadata.name,
            "mimeType": metadata.mime_type,
        });

        if let Some(parent_id) = options.parent_folder_id.as_ref() {
            file_metadata["parents"] = serde_json::json!([parent_id]);
        }

        // 4. Iniciar sessão de upload resumível
        debug!("Iniciando sessão de upload resumível...");
        let http_client = reqwest::Client::new();

        let init_response = http_client
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable")
            .bearer_auth(&token)
            .header("Content-Type", "application/json; charset=UTF-8")
            .json(&file_metadata)
            .send()
            .await
            .map_err(|e| RustDriveSyncError::UploadFailed {
                attempts: 1,
                message: format!("Erro ao iniciar upload resumível: {}", e),
            })?;

        // 5. Obter URL da sessão de upload
        let upload_url = init_response
            .headers()
            .get("Location")
            .ok_or_else(|| RustDriveSyncError::DriveApiError {
                message: "URL de upload não retornada pelo Google Drive".to_string(),
            })?
            .to_str()
            .map_err(|e| RustDriveSyncError::DriveApiError {
                message: format!("URL de upload inválida: {}", e),
            })?
            .to_string();

        debug!("URL de upload obtida: {}", upload_url);

        // 6. Abrir arquivo e criar stream
        let file = AsyncFile::open(&metadata.path).await.map_err(|e| {
            RustDriveSyncError::FileReadError {
                path: metadata.path.display().to_string(),
                message: e.to_string(),
            }
        })?;

        // Criar stream de 256KB chunks
        const CHUNK_SIZE: usize = 256 * 1024;
        let stream = ReaderStream::with_capacity(file, CHUNK_SIZE);

        // Converter stream para Body
        let body = reqwest::Body::wrap_stream(stream);

        // 7. Enviar arquivo
        debug!("Enviando arquivo ({} bytes)...", metadata.size);
        let upload_response = http_client
            .put(&upload_url)
            .header("Content-Length", metadata.size)
            .header("Content-Type", &metadata.mime_type)
            .body(body)
            .send()
            .await
            .map_err(|e| RustDriveSyncError::UploadFailed {
                attempts: 1,
                message: format!("Erro ao enviar arquivo: {}", e),
            })?;

        // 8. Processar resposta
        if !upload_response.status().is_success() {
            let error_text = upload_response.text().await.unwrap_or_default();
            return Err(RustDriveSyncError::UploadFailed {
                attempts: 1,
                message: format!("Upload falhou: {}", error_text),
            });
        }

        let uploaded_file: DriveFile = upload_response.json().await.map_err(|e| {
            RustDriveSyncError::DriveApiError {
                message: format!("Erro ao parsear resposta do upload: {}", e),
            }
        })?;

        debug!("Upload streaming concluído com sucesso!");

        Ok(UploadResult {
            file_id: uploaded_file.id.unwrap_or_default(),
            name: uploaded_file.name.unwrap_or_else(|| metadata.name.clone()),
            size: metadata.size,
            md5_checksum: Some(md5_hash),
            web_view_link: uploaded_file.web_view_link.clone(),
            upload_duration_secs: 0.0, // Será preenchido pelo caller
        })
    }

    /// Verifica se o checksum MD5 corresponde
    fn verify_checksum(&self, uploaded_file: &DriveFile, metadata: &LocalFileMetadata) -> Result<()> {
        if let (Some(remote_md5), Some(local_md5)) =
            (&uploaded_file.md5_checksum, &metadata.md5_hash)
        {
            if remote_md5 != local_md5 {
                warn!(
                    "Checksum MD5 não corresponde! Local: {}, Remoto: {}",
                    local_md5, remote_md5
                );
                return Err(RustDriveSyncError::ChecksumMismatch {
                    file: metadata.name.clone(),
                    expected: local_md5.clone(),
                    actual: remote_md5.clone(),
                });
            }
            debug!("Checksum MD5 verificado com sucesso");
        } else {
            debug!("Checksum MD5 não disponível para verificação");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google_drive::models::UploadProgress;

    #[test]
    fn test_mime_type_detection() {
        // Teste de detecção de MIME type usando mime_guess
        let test_cases = vec![
            ("test.txt", "text/plain"),
            ("image.jpg", "image/jpeg"),
            ("image.jpeg", "image/jpeg"),
            ("picture.png", "image/png"),
            ("unknown.unknown123", "application/octet-stream"), // Extensão realmente desconhecida
            ("document.pdf", "application/pdf"),
            ("data.json", "application/json"),
            ("video.mp4", "video/mp4"),
            ("audio.mp3", "audio/mpeg"),
            ("archive.zip", "application/zip"),
            ("page.html", "text/html"),
            ("style.css", "text/css"),
            ("script.js", "text/javascript"), // mime_guess usa text/javascript
            ("image.webp", "image/webp"), // Extensão moderna suportada
            ("data.xyz", "chemical/x-xyz"), // mime_guess conhece formato XYZ molecular
        ];

        for (filename, expected_mime) in test_cases {
            let detected = mime_guess::from_path(filename)
                .first_or_octet_stream()
                .to_string();

            assert_eq!(detected, expected_mime, "MIME type incorreto para {}", filename);
        }
    }

    #[test]
    fn test_upload_progress() {
        let progress = UploadProgress::new(500, 1000);
        assert_eq!(progress.percentage, 50.0);
        assert!(!progress.is_complete());

        let complete = UploadProgress::new(1000, 1000);
        assert_eq!(complete.percentage, 100.0);
        assert!(complete.is_complete());
    }
}
