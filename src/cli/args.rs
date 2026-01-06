use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// RustDriveSync - Sincronização unidirecional de arquivos com Google Drive
#[derive(Parser, Debug)]
#[command(name = "rustdrivesync")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Caminho para o arquivo de configuração
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,

    /// Modo verboso
    #[arg(short, long)]
    pub verbose: bool,

    /// Modo silencioso (apenas erros)
    #[arg(short, long)]
    pub quiet: bool,

    /// Subcomando a executar
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Executa sincronização
    Sync {
        /// Força execução única
        #[arg(long)]
        once: bool,

        /// Força modo monitoramento contínuo
        #[arg(long)]
        watch: bool,

        /// Simula execução sem fazer upload
        #[arg(long)]
        dry_run: bool,
    },

    /// Gerencia autenticação
    Auth {
        /// Revoga tokens existentes
        #[arg(long)]
        revoke: bool,

        /// Verifica status da autenticação
        #[arg(long)]
        status: bool,
    },

    /// Gerencia configuração
    Config {
        /// Cria arquivo de configuração exemplo
        #[arg(long)]
        init: bool,

        /// Valida arquivo de configuração
        #[arg(long)]
        validate: bool,

        /// Mostra configuração atual (sem secrets)
        #[arg(long)]
        show: bool,
    },

    /// Mostra status da última sincronização
    Status,

    /// Lista arquivos
    List {
        /// Lista arquivos no Google Drive
        #[arg(long)]
        remote: bool,

        /// Mostra diferenças local vs remoto
        #[arg(long)]
        diff: bool,
    },
}
