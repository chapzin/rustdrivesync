use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Inicializa o sistema de logging
///
/// Configura o tracing com níveis de log configuráveis via variável de ambiente
/// RUST_LOG ou diretamente através do parâmetro.
pub fn init_logger(log_level: &str) -> anyhow::Result<()> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_target(false).with_thread_ids(true))
        .try_init()
        .map_err(|e| anyhow::anyhow!("Falha ao inicializar logger: {}", e))?;

    Ok(())
}
