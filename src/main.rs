mod commands;
mod network;
mod security;
mod store;

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
        /// Path to a local environment file
        #[arg(long, conflicts_with = "ticket")]
        path: Option<String>,
        /// Iroh ticket for remote environment
        #[arg(long, conflicts_with = "path")]
        ticket: Option<String>,
        /// Target filename (used with --ticket, default: .env)
        #[arg(long)]
        filename: Option<String>,
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
    /// Shares an environment via P2P
    Share {
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

    // Initialize store
    let store = crate::store::iroh::IrohStore::new().await?;

    let passphrase_input = match &cli.command {
        Commands::List { .. } | Commands::Share { .. } => None, // Not needed for listing or sharing
        Commands::Register {
            ticket: Some(_), ..
        } => None, // Not needed for remote registration (blob already encrypted)
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
                commands::SetArgs {
                    project: &project,
                    env: &env,
                    force,
                    passphrase: passphrase.expect("Passphrase required for set"),
                },
                &store,
            )
            .await?
        }
        Commands::Register {
            project,
            env,
            path,
            ticket,
            filename,
            force,
        } => {
            let source = if let Some(p) = path {
                commands::RegisterSource::Local { path: p }
            } else if let Some(t) = ticket {
                commands::RegisterSource::Remote {
                    ticket: t,
                    filename,
                }
            } else {
                anyhow::bail!("Must provide either --path or --ticket");
            };

            let args = commands::RegisterArgs {
                project: &project,
                env: &env,
                force,
                passphrase,
            };

            commands::register(args, source, &store).await?
        }
        Commands::Rm {
            project,
            env,
            force,
        } => {
            commands::remove(
                commands::RmArgs {
                    project: &project,
                    env: &env,
                    force,
                    passphrase: passphrase.expect("Passphrase required for rm"),
                },
                &store,
            )
            .await?
        }
        Commands::List { project } => {
            commands::list(
                commands::ListArgs {
                    project: project.as_deref(),
                },
                &store,
            )
            .await?
        }
        Commands::Show { project, env } => {
            commands::show(
                commands::ShowArgs {
                    project: &project,
                    env: &env,
                    passphrase: passphrase.expect("Passphrase required for show"),
                },
                &store,
            )
            .await?
        }
        Commands::Share { project, env } => {
            commands::share(
                commands::ShareArgs {
                    project: &project,
                    env: &env,
                },
                &store,
            )
            .await?
        }
    }

    Ok(())
}
