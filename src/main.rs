use envbro::commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use secrecy::SecretString;

#[derive(Parser)]
#[command(name = "envbro")]
#[command(about = "Drugs for your env files insecurities")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sets the environment for a project
    Set {
        /// Project name
        project: String,
        /// Environment to set
        env: String,
        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,
    },
    /// Adds a new environment to a project
    Register {
        /// Project name
        project: String,
        /// Environment name
        env: String,
        /// Path to the environment file
        #[arg(long)]
        path: String,
        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,
    },
    /// Removes an environment from a project
    Rm {
        /// Project name
        project: String,
        /// Environment name
        env: String,
        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,
    },
    /// Lists environments
    List {
        /// Filter by project
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Shows environment file contents
    Show {
        /// Project name
        project: String,
        /// Environment name
        env: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    let passphrase_input = match &cli.command {
        Commands::List { .. } => None,
        _ => {
            if let Ok(p) = std::env::var("ENVBRO_PASSPHRASE") {
                Some(SecretString::from(p))
            } else {
                Some(SecretString::from(
                    dialoguer::Password::new()
                        .with_prompt("Passphrase")
                        .interact()?,
                ))
            }
        }
    };

    let passphrase = passphrase_input.as_ref();

    match cli.command {
        Commands::Set {
            project,
            env,
            force,
        } => {
            commands::set_env(
                &project,
                &env,
                force,
                passphrase.expect("Passphrase required for set"),
            )
            .await?
        }
        Commands::Register {
            project,
            env,
            path,
            force,
        } => {
            commands::register(
                &project,
                &env,
                &path,
                force,
                passphrase.expect("Passphrase required for register"),
            )
            .await?
        }
        Commands::Rm {
            project,
            env,
            force,
        } => {
            commands::remove(
                &project,
                &env,
                force,
                passphrase.expect("Passphrase required for rm"),
            )
            .await?
        }
        Commands::List { project } => commands::list(project.as_deref()).await?,
        Commands::Show { project, env } => {
            commands::show(
                &project,
                &env,
                passphrase.expect("Passphrase required for show"),
            )
            .await?
        }
    }

    Ok(())
}
