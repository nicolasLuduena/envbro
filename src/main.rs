mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

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

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Set {
            project,
            env,
            force,
        } => commands::set_env(&project, &env, force)?,
        Commands::Register {
            project,
            env,
            path,
            force,
        } => commands::register(&project, &env, &path, force)?,
        Commands::Rm {
            project,
            env,
            force,
        } => commands::remove(&project, &env, force)?,
        Commands::List { project } => commands::list(project.as_deref())?,
        Commands::Show { project, env } => commands::show(&project, &env)?,
    }

    Ok(())
}
