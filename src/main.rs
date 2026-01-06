use clap::Parser;
use rustdrivesync::cli::{execute, Cli};
use rustdrivesync::logging::init_logger;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Parse argumentos da linha de comando
    let cli = Cli::parse();

    // Configurar nível de log
    let log_level = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    };

    // Inicializar logger
    if let Err(e) = init_logger(log_level) {
        eprintln!("Falha ao inicializar logger: {}", e);
        std::process::exit(1);
    }

    info!("RustDriveSync v{}", env!("CARGO_PKG_VERSION"));

    // Executar comando
    if let Err(e) = execute(cli).await {
        error!("Erro: {}", e);
        std::process::exit(1);
    }

    info!("Execução concluída com sucesso");
}
