use crate::config::Config;
use crate::error::{Result, RustDriveSyncError};
use crate::google_drive::auth::DriveAuthenticator;
use crate::google_drive::client::DriveClient;
use crate::google_drive::models::{UploadOptions, UploadResult};
use crate::google_drive::upload::DriveUploader;
use crate::sync::scanner::{FileScanner, LocalFile};
use crate::sync::state::{FileState, SyncStateManager};
use crate::sync::tracker::{ChangeTracker, ChangeType, FileChange};
use crate::watcher::{FileEvent, FileWatcher, WatcherConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Modo de sincronização
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncMode {
    /// Sincronização única e depois para
    Once,
    /// Monitoramento contínuo de mudanças
    Watch,
}

/// Estatísticas de sincronização
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    /// Total de arquivos escaneados
    pub files_scanned: usize,
    /// Arquivos novos enviados
    pub files_uploaded: usize,
    /// Arquivos atualizados
    pub files_updated: usize,
    /// Arquivos que falharam
    pub files_failed: usize,
    /// Bytes totais enviados
    pub bytes_uploaded: u64,
    /// Tempo total de sincronização (segundos)
    pub duration_secs: f64,
}

impl SyncStats {
    /// Incrementa contador de uploads bem-sucedidos
    pub fn record_upload(&mut self, size: u64, is_new: bool) {
        if is_new {
            self.files_uploaded += 1;
        } else {
            self.files_updated += 1;
        }
        self.bytes_uploaded += size;
    }

    /// Incrementa contador de falhas
    pub fn record_failure(&mut self) {
        self.files_failed += 1;
    }

    /// Formata estatísticas para exibição
    pub fn summary(&self) -> String {
        format!(
            "Escaneados: {}, Novos: {}, Atualizados: {}, Falhas: {}, Bytes: {}, Tempo: {:.2}s",
            self.files_scanned,
            self.files_uploaded,
            self.files_updated,
            self.files_failed,
            self.bytes_uploaded,
            self.duration_secs
        )
    }
}

/// Resultado de uma sincronização
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Estatísticas da sincronização
    pub stats: SyncStats,
    /// Sucesso geral
    pub success: bool,
    /// Mensagem de erro (se houver)
    pub error_message: Option<String>,
}

/// Engine de sincronização
pub struct SyncEngine {
    /// Configuração
    config: Config,
    /// Cliente do Google Drive
    drive_client: DriveClient,
    /// Scanner de arquivos
    file_scanner: FileScanner,
    /// Gerenciador de estado
    state_manager: SyncStateManager,
    /// Modo de sincronização
    mode: SyncMode,
    /// Modo dry-run (não faz uploads reais)
    dry_run: bool,
    /// Flag de shutdown para modo watch
    shutdown: Arc<AtomicBool>,
}

impl SyncEngine {
    /// Cria um novo engine de sincronização
    pub async fn new(config: Config, mode: SyncMode, dry_run: bool) -> Result<Self> {
        info!("Inicializando engine de sincronização");

        // Criar autenticador
        let auth = DriveAuthenticator::new(
            &config.google_drive.credentials_file,
            &config.google_drive.token_file,
            config.google_drive.scopes.clone(),
        )
        .await?;

        // Criar cliente Drive
        let drive_client = DriveClient::new(auth).await?;

        // Garantir que a pasta de destino existe
        let folder_name = config
            .google_drive
            .target_folder_name
            .clone()
            .unwrap_or_else(|| "RustDriveSync".to_string());

        let drive_folder = drive_client.ensure_folder(&folder_name, None).await?;

        info!(
            "Pasta do Drive: {} (ID: {})",
            drive_folder.name, drive_folder.id
        );

        // Configurar scanner
        let max_file_size = if config.sync.max_file_size_mb > 0 {
            Some(config.sync.max_file_size_mb * 1024 * 1024)
        } else {
            None
        };

        let file_scanner = FileScanner::new()
            .with_ignore_patterns(config.source.ignore_patterns.clone())
            .with_max_file_size(max_file_size.unwrap_or(u64::MAX));

        // Configurar gerenciador de estado
        let state_file_path = config.state.state_file.clone();
        let state_manager = SyncStateManager::load_or_create(
            state_file_path,
            config.source.path.display().to_string(),
            drive_folder.id.clone(),
        )?;

        Ok(Self {
            config,
            drive_client,
            file_scanner,
            state_manager,
            mode,
            dry_run,
            shutdown: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Retorna um clone do shutdown flag para sinalização externa
    pub fn shutdown_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown)
    }

    /// Sinaliza o shutdown do engine
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
        info!("Shutdown solicitado");
    }

    /// Verifica se o shutdown foi solicitado
    fn should_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    /// Executa uma sincronização completa
    pub async fn sync_once(&mut self) -> Result<SyncResult> {
        let start_time = std::time::Instant::now();
        let mut stats = SyncStats::default();

        info!("Iniciando sincronização única");

        // 1. Escanear diretório local
        info!("Escaneando diretório: {}", self.config.source.path.display());
        let local_files = self
            .file_scanner
            .scan_directory(&self.config.source.path)?;

        stats.files_scanned = local_files.len();
        info!("Encontrados {} arquivos", local_files.len());

        // 2. Detectar mudanças
        let changes = ChangeTracker::detect_changes(&local_files, self.state_manager.state());
        let sync_needed = ChangeTracker::filter_sync_needed(changes);

        info!("Arquivos a sincronizar: {}", sync_needed.len());

        // 3. Sincronizar arquivos
        for change in sync_needed {
            match self.sync_file_change(&change, &mut stats).await {
                Ok(_) => {
                    debug!("Arquivo sincronizado: {}", change.relative_path);
                }
                Err(e) => {
                    error!("Erro ao sincronizar {}: {}", change.relative_path, e);
                    stats.record_failure();
                }
            }
        }

        // 4. Marcar sincronização completa
        if stats.files_failed == 0 {
            self.state_manager.mark_full_sync_and_save()?;
        }

        stats.duration_secs = start_time.elapsed().as_secs_f64();

        info!("Sincronização concluída: {}", stats.summary());

        Ok(SyncResult {
            success: stats.files_failed == 0,
            stats,
            error_message: None,
        })
    }

    /// Executa sincronização em modo watch (contínuo)
    pub async fn sync_watch(&mut self) -> Result<SyncResult> {
        let start_time = std::time::Instant::now();

        info!("🔍 Iniciando modo watch - monitoramento contínuo de arquivos");

        // 1. Fazer uma sincronização inicial completa
        info!("Executando sincronização inicial...");
        let initial_result = self.sync_once().await?;
        let mut stats = initial_result.stats;

        info!(
            "Sincronização inicial concluída: {}",
            stats.summary()
        );

        // 2. Criar file watcher
        let watcher_config = WatcherConfig::default()
            .with_ignore_patterns(self.config.source.ignore_patterns.clone())
            .with_debounce(Duration::from_secs(2));

        let watcher = FileWatcher::new(&self.config.source.path, watcher_config)?;

        info!(
            "👁️  Monitorando mudanças em: {}",
            self.config.source.path.display()
        );
        info!("Pressione Ctrl+C para parar...");

        // 3. Loop de monitoramento
        loop {
            // Verificar shutdown
            if self.should_shutdown() {
                info!("Shutdown detectado, encerrando modo watch");
                break;
            }

            // Aguardar próximo evento com timeout
            match self.wait_for_event_with_timeout(&watcher, Duration::from_secs(1)) {
                Some(event) => {
                    if event.needs_sync() {
                        info!(
                            "📝 Mudança detectada: {:?} - {}",
                            event.event_type,
                            event.path.display()
                        );

                        // Processar evento
                        match self.process_watch_event(event, &mut stats).await {
                            Ok(_) => {
                                info!("✅ Arquivo sincronizado com sucesso");
                            }
                            Err(e) => {
                                error!("❌ Erro ao sincronizar arquivo: {}", e);
                                stats.record_failure();
                            }
                        }
                    } else {
                        debug!("Evento ignorado: {:?}", event.event_type);
                    }
                }
                None => {
                    // Timeout - continuar loop
                    continue;
                }
            }
        }

        stats.duration_secs = start_time.elapsed().as_secs_f64();

        info!("Modo watch encerrado: {}", stats.summary());

        Ok(SyncResult {
            success: stats.files_failed == 0,
            stats,
            error_message: None,
        })
    }

    /// Aguarda evento com timeout
    fn wait_for_event_with_timeout(
        &self,
        watcher: &FileWatcher,
        timeout: Duration,
    ) -> Option<FileEvent> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() >= timeout {
                return None;
            }

            if let Some(event) = watcher.try_next_event() {
                return Some(event);
            }

            // Pequeno sleep para não consumir CPU
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Processa um evento do file watcher
    async fn process_watch_event(
        &mut self,
        event: FileEvent,
        stats: &mut SyncStats,
    ) -> Result<()> {
        // Verificar se o arquivo ainda existe (pode ter sido deletado rapidamente)
        if !event.path.exists() {
            debug!("Arquivo não existe mais, ignorando: {}", event.path.display());
            return Ok(());
        }

        // Verificar se é um diretório
        if event.path.is_dir() {
            debug!("Ignorando diretório: {}", event.path.display());
            return Ok(());
        }

        // Criar LocalFile a partir do evento
        let local_file = LocalFile::from_path(
            event.path.clone(),
            &self.config.source.path,
        )?;

        // Verificar se deve ser ignorado pelo scanner
        if let Some(max_size) = self.file_scanner.config().max_file_size {
            if local_file.size > max_size {
                warn!(
                    "Arquivo muito grande ({}), ignorando: {}",
                    local_file.size,
                    event.path.display()
                );
                return Ok(());
            }
        }

        // Determinar se é novo ou modificado
        let relative_path = local_file.relative_path.display().to_string();
        let is_new = !self.state_manager.state().is_synced(&relative_path);

        // Criar FileChange
        let change = if is_new {
            FileChange::new(local_file)
        } else {
            let previous_state = self
                .state_manager
                .state()
                .get_file(&relative_path)
                .cloned()
                .ok_or_else(|| RustDriveSyncError::StateError {
                    message: format!(
                        "Arquivo não encontrado no estado: {}",
                        relative_path
                    ),
                })?;
            FileChange::modified(local_file, previous_state)
        };

        // Sincronizar
        self.sync_file_change(&change, stats).await?;

        Ok(())
    }

    /// Sincroniza uma mudança de arquivo
    async fn sync_file_change(
        &mut self,
        change: &FileChange,
        stats: &mut SyncStats,
    ) -> Result<()> {
        let local_file = change
            .local_file
            .as_ref()
            .ok_or_else(|| RustDriveSyncError::DriveApiError {
                message: "Arquivo local não disponível".to_string(),
            })?;

        info!(
            "Sincronizando: {} ({} bytes)",
            change.relative_path, local_file.size
        );

        if self.dry_run {
            info!("[DRY-RUN] Pulando upload real");
            stats.record_upload(local_file.size, change.change_type == ChangeType::New);
            return Ok(());
        }

        // Criar uploader
        let uploader = DriveUploader::new(&self.drive_client);

        // Configurar opções de upload
        let upload_options = UploadOptions::with_parent(
            self.state_manager.state().drive_folder_id.clone(),
        );

        // Fazer upload
        let upload_result = uploader
            .upload_file(&local_file.path, upload_options)
            .await?;

        // Atualizar estado
        let file_state = self.create_file_state(local_file, &upload_result)?;
        self.state_manager.update_and_save(file_state)?;

        stats.record_upload(local_file.size, change.change_type == ChangeType::New);

        info!(
            "Upload concluído: {} -> {}",
            change.relative_path, upload_result.file_id
        );

        Ok(())
    }

    /// Cria um FileState a partir do resultado do upload
    fn create_file_state(
        &self,
        local_file: &LocalFile,
        upload_result: &UploadResult,
    ) -> Result<FileState> {
        Ok(FileState {
            relative_path: local_file.relative_path.display().to_string(),
            drive_file_id: upload_result.file_id.clone(),
            size: upload_result.size,
            modified: local_file.modified,
            md5_hash: upload_result
                .md5_checksum
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            last_synced: chrono::Utc::now().timestamp(),
        })
    }

    /// Executa sincronização de acordo com o modo configurado
    pub async fn run(&mut self) -> Result<SyncResult> {
        match self.mode {
            SyncMode::Once => self.sync_once().await,
            SyncMode::Watch => self.sync_watch().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_stats() {
        let mut stats = SyncStats::default();
        assert_eq!(stats.files_uploaded, 0);

        stats.record_upload(1024, true);
        assert_eq!(stats.files_uploaded, 1);
        assert_eq!(stats.bytes_uploaded, 1024);

        stats.record_upload(2048, false);
        assert_eq!(stats.files_updated, 1);
        assert_eq!(stats.bytes_uploaded, 3072);

        stats.record_failure();
        assert_eq!(stats.files_failed, 1);
    }

    #[test]
    fn test_sync_mode() {
        assert_eq!(SyncMode::Once, SyncMode::Once);
        assert_ne!(SyncMode::Once, SyncMode::Watch);
    }
}
