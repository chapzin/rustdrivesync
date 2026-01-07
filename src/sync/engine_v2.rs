use crate::config::Config;
use crate::core::retry::{retry_with_backoff, RetryConfig};
use crate::error::{Result, RustDriveSyncError};
use crate::google_drive::auth::DriveAuthenticator;
use crate::google_drive::client::DriveClient;
use crate::google_drive::DriveStorageBackend;
use crate::storage::{StorageBackend, UploadOptions};
use crate::sync::scanner::{FileScanner, LocalFile};
use crate::sync::state::{FileState, SyncStateManager};
use crate::sync::tracker::{ChangeTracker, ChangeType, FileChange};
use crate::watcher::{FileEvent, FileWatcher, WatcherConfig};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
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

/// Engine de sincronização com suporte a diferentes backends via trait
pub struct SyncEngine<S: StorageBackend> {
    /// Configuração
    config: Config,
    /// Backend de storage (abstrato)
    storage_backend: Arc<S>,
    /// Scanner de arquivos
    file_scanner: FileScanner,
    /// Gerenciador de estado (thread-safe)
    state_manager: Arc<Mutex<SyncStateManager>>,
    /// Modo de sincronização
    mode: SyncMode,
    /// Modo dry-run (não faz uploads reais)
    dry_run: bool,
    /// Flag de shutdown para modo watch
    shutdown: Arc<AtomicBool>,
    /// Configuração de retry
    retry_config: RetryConfig,
    /// Semáforo para controlar uploads concorrentes
    upload_semaphore: Arc<Semaphore>,
}

impl SyncEngine<DriveStorageBackend> {
    /// Cria um novo engine de sincronização com Google Drive (factory method)
    ///
    /// # Argumentos
    /// * `config` - Configuração carregada do arquivo TOML
    /// * `mode` - Modo de operação (Once ou Watch)
    /// * `dry_run` - Se true, não faz uploads reais
    ///
    /// # Erros
    /// Retorna erro se:
    /// - Falhar autenticação com Google Drive
    /// - Pasta de destino não existir
    /// - State file corrompido
    pub async fn new(config: Config, mode: SyncMode, dry_run: bool) -> Result<Self> {
        info!("Inicializando engine de sincronização com Google Drive");

        // Criar autenticador
        let auth = DriveAuthenticator::new(
            &config.google_drive.credentials_file,
            &config.google_drive.token_file,
            config.google_drive.scopes.clone(),
        )
        .await?;

        // Criar cliente Drive
        let drive_client = DriveClient::new(auth).await?;

        // Criar backend de storage com rate limiting
        let storage_backend = DriveStorageBackend::new(drive_client);

        // Garantir que a pasta de destino existe
        let folder_name = config
            .google_drive
            .target_folder_name
            .clone()
            .unwrap_or_else(|| "RustDriveSync".to_string());

        let drive_folder = storage_backend.ensure_folder(&folder_name, None).await?;

        info!(
            "Pasta do Drive: {} (ID: {})",
            drive_folder.name, drive_folder.id
        );

        // Criar engine com backend
        Self::with_backend(config, Arc::new(storage_backend), drive_folder.id, mode, dry_run)
    }
}

impl<S: StorageBackend + 'static> SyncEngine<S> {
    /// Cria um engine com um backend customizado (para testes e outros storages)
    ///
    /// # Argumentos
    /// * `config` - Configuração
    /// * `storage_backend` - Backend de storage (Google Drive, S3, Mock, etc)
    /// * `target_folder_id` - ID da pasta de destino
    /// * `mode` - Modo de sincronização
    /// * `dry_run` - Se true, simula uploads sem enviar
    pub fn with_backend(
        config: Config,
        storage_backend: Arc<S>,
        target_folder_id: String,
        mode: SyncMode,
        dry_run: bool,
    ) -> Result<Self> {
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
            target_folder_id,
        )?;

        // Configurar retry a partir da config
        let retry_config = RetryConfig {
            max_attempts: config.retry.max_attempts,
            initial_delay_secs: config.retry.initial_delay_seconds,
            backoff_multiplier: config.retry.backoff_multiplier,
            max_delay_secs: config.retry.max_delay_seconds,
        };

        info!("Retry configurado: {:?}", retry_config);

        // Configurar semáforo para uploads concorrentes
        let max_concurrent = config.sync.max_concurrent_uploads;
        info!(
            "Uploads concorrentes configurados: {} simultâneos",
            max_concurrent
        );

        Ok(Self {
            config,
            storage_backend,
            file_scanner,
            state_manager: Arc::new(Mutex::new(state_manager)),
            mode,
            dry_run,
            shutdown: Arc::new(AtomicBool::new(false)),
            retry_config,
            upload_semaphore: Arc::new(Semaphore::new(max_concurrent)),
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
        let state_guard = self.state_manager.lock().await;
        let changes = ChangeTracker::detect_changes(&local_files, state_guard.state());
        let sync_needed = ChangeTracker::filter_sync_needed(changes);
        drop(state_guard); // Liberar lock antes de sincronizar

        info!("Arquivos a sincronizar: {}", sync_needed.len());

        // 3. Sincronizar arquivos em paralelo
        let upload_stats = self.sync_files_parallel(sync_needed).await;

        // Consolidar estatísticas
        stats.files_uploaded = upload_stats.files_uploaded;
        stats.files_updated = upload_stats.files_updated;
        stats.files_failed = upload_stats.files_failed;
        stats.bytes_uploaded = upload_stats.bytes_uploaded;

        // 4. Marcar sincronização completa
        if stats.files_failed == 0 {
            let mut state_guard = self.state_manager.lock().await;
            state_guard.mark_full_sync_and_save()?;
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

    /// Sincroniza múltiplos arquivos em paralelo
    ///
    /// Usa tokio::spawn para processar uploads concorrentemente,
    /// controlado pelo semáforo para limitar concorrência.
    async fn sync_files_parallel(&self, changes: Vec<FileChange>) -> SyncStats {
        if changes.is_empty() {
            return SyncStats::default();
        }

        let total_files = changes.len();
        info!(
            "🚀 Iniciando sincronização paralela de {} arquivos",
            total_files
        );

        // Contadores atômicos para estatísticas
        let uploaded_count = Arc::new(AtomicUsize::new(0));
        let updated_count = Arc::new(AtomicUsize::new(0));
        let failed_count = Arc::new(AtomicUsize::new(0));
        let bytes_uploaded = Arc::new(AtomicUsize::new(0));

        // Spawnar tasks para cada arquivo
        let mut tasks = Vec::new();

        for change in changes {
            let storage = Arc::clone(&self.storage_backend);
            let state_manager = Arc::clone(&self.state_manager);
            let retry_config = self.retry_config.clone();
            let semaphore = Arc::clone(&self.upload_semaphore);
            let dry_run = self.dry_run;

            // Contadores clonados
            let uploaded = Arc::clone(&uploaded_count);
            let updated = Arc::clone(&updated_count);
            let failed = Arc::clone(&failed_count);
            let bytes = Arc::clone(&bytes_uploaded);

            let task = tokio::spawn(async move {
                // Adquirir permissão do semáforo
                let _permit = semaphore.acquire().await.expect("Semaphore closed");

                match Self::sync_single_file(
                    &change,
                    &storage,
                    &state_manager,
                    retry_config,
                    dry_run,
                )
                .await
                {
                    Ok(size) => {
                        let is_new = change.change_type == ChangeType::New;
                        if is_new {
                            uploaded.fetch_add(1, Ordering::Relaxed);
                        } else {
                            updated.fetch_add(1, Ordering::Relaxed);
                        }
                        bytes.fetch_add(size as usize, Ordering::Relaxed);
                        debug!("✅ Sincronizado: {}", change.relative_path);
                    }
                    Err(e) => {
                        failed.fetch_add(1, Ordering::Relaxed);
                        error!("❌ Falha ao sincronizar {}: {}", change.relative_path, e);
                    }
                }
            });

            tasks.push(task);
        }

        // Aguardar todas as tasks
        info!("⏳ Aguardando conclusão de {} uploads paralelos...", total_files);
        let results = futures::future::join_all(tasks).await;

        // Contar quantas tasks falharam (panic ou join error)
        let task_failures = results.iter().filter(|r| r.is_err()).count();
        if task_failures > 0 {
            warn!("{} tasks falharam com panic", task_failures);
        }

        // Construir estatísticas
        let stats = SyncStats {
            files_scanned: 0, // Será preenchido pelo caller
            files_uploaded: uploaded_count.load(Ordering::Relaxed),
            files_updated: updated_count.load(Ordering::Relaxed),
            files_failed: failed_count.load(Ordering::Relaxed) + task_failures,
            bytes_uploaded: bytes_uploaded.load(Ordering::Relaxed) as u64,
            duration_secs: 0.0, // Será preenchido pelo caller
        };

        info!(
            "✅ Sincronização paralela concluída: {} novos, {} atualizados, {} falhas",
            stats.files_uploaded, stats.files_updated, stats.files_failed
        );

        stats
    }

    /// Sincroniza um único arquivo (versão estática para uso em tasks)
    async fn sync_single_file<B: StorageBackend + 'static>(
        change: &FileChange,
        storage: &Arc<B>,
        state_manager: &Arc<Mutex<SyncStateManager>>,
        retry_config: RetryConfig,
        dry_run: bool,
    ) -> Result<u64> {
        let local_file = change
            .local_file
            .as_ref()
            .ok_or_else(|| RustDriveSyncError::DriveApiError {
                message: "Arquivo local não disponível".to_string(),
            })?;

        debug!(
            "Sincronizando: {} ({} bytes)",
            change.relative_path, local_file.size
        );

        if dry_run {
            debug!("[DRY-RUN] Pulando upload real de {}", change.relative_path);
            return Ok(local_file.size);
        }

        // Obter folder_id do estado
        let folder_id = {
            let guard = state_manager.lock().await;
            guard.state().drive_folder_id.clone()
        };

        // Configurar opções de upload
        let upload_options = UploadOptions::with_parent(folder_id);

        // Fazer upload com retry automático
        let file_path = local_file.path.clone();
        let relative_path = change.relative_path.clone();

        let upload_result = retry_with_backoff(
            retry_config,
            || {
                let storage_ref = Arc::clone(storage);
                let path = file_path.clone();
                let opts = upload_options.clone();
                async move { storage_ref.upload_file(&path, opts).await }
            },
            &format!("upload_{}", relative_path),
        )
        .await?;

        // Atualizar estado
        let file_state = FileState {
            relative_path: local_file.relative_path.display().to_string(),
            drive_file_id: upload_result.file_id.clone(),
            size: upload_result.size,
            modified: local_file.modified,
            md5_hash: upload_result
                .md5_checksum
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            last_synced: chrono::Utc::now().timestamp(),
        };

        let mut guard = state_manager.lock().await;
        guard.update_and_save(file_state)?;

        debug!(
            "Upload concluído: {} -> {}",
            relative_path, upload_result.file_id
        );

        Ok(local_file.size)
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

        let state_guard = self.state_manager.lock().await;
        let is_new = !state_guard.state().is_synced(&relative_path);

        // Criar FileChange
        let change = if is_new {
            drop(state_guard);
            FileChange::new(local_file)
        } else {
            let previous_state = state_guard
                .state()
                .get_file(&relative_path)
                .cloned()
                .ok_or_else(|| RustDriveSyncError::StateError {
                    message: format!(
                        "Arquivo não encontrado no estado: {}",
                        relative_path
                    ),
                })?;
            drop(state_guard);
            FileChange::modified(local_file, previous_state)
        };

        // Sincronizar (reutiliza a função estática)
        match Self::sync_single_file(
            &change,
            &self.storage_backend,
            &self.state_manager,
            self.retry_config.clone(),
            self.dry_run,
        )
        .await
        {
            Ok(size) => {
                stats.record_upload(size, change.change_type == ChangeType::New);
            }
            Err(e) => {
                error!("Erro ao sincronizar: {}", e);
                stats.record_failure();
                return Err(e);
            }
        }

        Ok(())
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

    #[test]
    fn test_retry_config_from_app_config() {
        let retry = crate::config::schema::RetryConfig::default();
        assert_eq!(retry.max_attempts, 3);
    }
}
