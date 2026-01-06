use crate::cli::args::{Cli, Commands};
use crate::config::{load_config, validate_config};
use crate::error::Result;
use crate::google_drive::auth;
use crate::sync::engine::{SyncEngine, SyncMode};
use tracing::{info, warn};

/// Executa o comando solicitado
pub async fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Sync {
            once,
            watch,
            dry_run,
        } => {
            info!("Iniciando sincronização...");
            if dry_run {
                warn!("Modo dry-run ativado - nenhum arquivo será enviado");
            }

            let config = load_config(&cli.config)?;
            validate_config(&config)?;

            // Determinar modo de sincronização
            let mode = if once {
                SyncMode::Once
            } else if watch {
                SyncMode::Watch
            } else {
                match config.sync.mode.as_str() {
                    "watch" => SyncMode::Watch,
                    _ => SyncMode::Once,
                }
            };

            info!("Modo de sincronização: {:?}", mode);

            // Criar e executar engine
            let mut engine = SyncEngine::new(config, mode.clone(), dry_run).await?;

            // Se for modo watch, configurar handler de Ctrl+C
            if mode == SyncMode::Watch {
                let shutdown_handle = engine.shutdown_handle();
                ctrlc::set_handler(move || {
                    println!("\n🛑 Ctrl+C recebido, encerrando...");
                    shutdown_handle.store(true, std::sync::atomic::Ordering::Relaxed);
                })
                .expect("Erro ao configurar handler de Ctrl+C");
            }

            let result = engine.run().await?;

            // Exibir resultado
            println!("\n📊 Resultado da Sincronização");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            println!("Arquivos escaneados:  {}", result.stats.files_scanned);
            println!("Arquivos novos:       {}", result.stats.files_uploaded);
            println!("Arquivos atualizados: {}", result.stats.files_updated);
            println!("Falhas:               {}", result.stats.files_failed);
            println!("Bytes enviados:       {} ({:.2} MB)",
                result.stats.bytes_uploaded,
                result.stats.bytes_uploaded as f64 / 1_048_576.0
            );
            println!("Tempo total:          {:.2}s", result.stats.duration_secs);

            if result.success {
                println!("\n✅ Sincronização concluída com sucesso!");
            } else {
                println!("\n⚠️  Sincronização concluída com erros");
            }

            Ok(())
        }

        Commands::Auth { revoke, status } => {
            // Carregar configuração para obter paths e scopes
            let config = load_config(&cli.config)?;

            if revoke {
                info!("Revogando tokens...");
                auth::revoke_token(&config.google_drive.token_file).await?;
            } else if status {
                info!("Verificando status da autenticação...");
                auth::check_auth_status(
                    &config.google_drive.credentials_file,
                    &config.google_drive.token_file,
                    config.google_drive.scopes.clone(),
                )
                .await?;
            } else {
                info!("Iniciando fluxo de autenticação...");
                println!("\n🔐 Autenticação Google Drive OAuth2");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                println!("1. Uma janela do navegador será aberta");
                println!("2. Faça login com sua conta Google");
                println!("3. Autorize o RustDriveSync");
                println!("4. O token será salvo automaticamente\n");

                auth::authenticate(
                    &config.google_drive.credentials_file,
                    &config.google_drive.token_file,
                    config.google_drive.scopes.clone(),
                )
                .await?;
            }

            Ok(())
        }

        Commands::Config {
            init,
            validate,
            show,
        } => {
            if init {
                info!("Criando arquivo de configuração exemplo...");
                create_example_config()?;
                info!("Arquivo config.example.toml criado com sucesso!");
            } else if validate {
                info!("Validando configuração...");
                let config = load_config(&cli.config)?;
                validate_config(&config)?;
                info!("Configuração válida!");
            } else if show {
                info!("Mostrando configuração...");
                let config = load_config(&cli.config)?;
                println!("{:#?}", config);
            }

            Ok(())
        }

        Commands::Status => {
            info!("Mostrando status da sincronização...");
            // TODO: Implementar exibição de status
            Ok(())
        }

        Commands::List { remote, diff } => {
            info!("Listando arquivos...");

            if remote {
                info!("Listando arquivos no Google Drive...");
                // TODO: Implementar listagem remota
            } else if diff {
                info!("Mostrando diferenças local vs remoto...");
                // TODO: Implementar diff
            } else {
                info!("Listando arquivos pendentes...");
                // TODO: Implementar listagem local
            }

            Ok(())
        }
    }
}

fn create_example_config() -> Result<()> {
    let example_config = include_str!("../../config.example.toml");
    std::fs::write("config.example.toml", example_config)?;
    Ok(())
}
