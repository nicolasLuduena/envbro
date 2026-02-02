use crate::security;
use crate::store::{SharableStore, Store};
use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use secrecy::SecretString;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

pub enum RegisterSource {
    Local {
        path: String,
    },
    Remote {
        ticket: String,
        filename: Option<String>,
    },
}

pub struct RegisterArgs<'a> {
    pub project: &'a str,
    pub env: &'a str,
    pub force: bool,
    pub passphrase: Option<&'a SecretString>,
}

pub struct SetArgs<'a> {
    pub project: &'a str,
    pub env: &'a str,
    pub force: bool,
    pub passphrase: &'a SecretString,
}

pub struct RmArgs<'a> {
    pub project: &'a str,
    pub env: &'a str,
    pub force: bool,
    pub passphrase: &'a SecretString,
}

pub struct ListArgs<'a> {
    pub project: Option<&'a str>,
}

pub struct ShowArgs<'a> {
    pub project: &'a str,
    pub env: &'a str,
    pub passphrase: &'a SecretString,
}

pub struct ShareArgs<'a> {
    pub project: &'a str,
    pub env: &'a str,
}

pub async fn register(
    args: RegisterArgs<'_>,
    source: RegisterSource,
    store: &impl Store,
) -> Result<()> {
    if args.env.contains('/') {
        bail!("Environment name cannot contain /");
    }

    // Determine the filename and content source
    let (target_filename, encrypted_content_bytes) = match source {
        RegisterSource::Local { path } => {
            // Local registration: read from file
            let passphrase = args
                .passphrase
                .context("Passphrase required for local registration")?;

            let target_path = Path::new(&path);
            if !target_path.exists() || !target_path.is_file() {
                bail!("Target file doesn't exist or is not a file");
            }

            let new_content_bytes = fs::read(target_path).context("Failed to read target file")?;
            // Verify it is UTF-8
            let _ = std::str::from_utf8(&new_content_bytes)
                .context("Target file is not valid UTF-8")?;

            let filename = target_path
                .file_name()
                .context("Invalid target filename")?
                .to_string_lossy()
                .to_string();

            let encrypted = security::encrypt(&new_content_bytes, passphrase)?;
            (filename, encrypted)
        }
        RegisterSource::Remote { ticket, filename } => {
            // Remote registration: download from P2P
            let filename = filename.unwrap_or_else(|| ".env".to_string());

            let (hash, encrypted_bytes) = crate::network::fetch_from_ticket(&ticket).await?;

            info!("Downloaded blob with hash: {}", hash);

            (filename, encrypted_bytes)
        }
    };

    // Check existing and show diff if applicable
    if let Ok((_, existing_encrypted)) = store.get(args.project, args.env).await {
        if args.passphrase.is_some() {
            // Only show diff if we have a passphrase to decrypt
            let old_content_bytes =
                security::decrypt(&existing_encrypted, args.passphrase.unwrap())
                    .context("Failed to decrypt existing env file")?;
            let old_content = String::from_utf8(old_content_bytes)
                .context("Stored content is not valid UTF-8")?;

            let new_content_bytes =
                security::decrypt(&encrypted_content_bytes, args.passphrase.unwrap())
                    .context("Failed to decrypt new content")?;
            let new_content =
                String::from_utf8(new_content_bytes).context("New content is not valid UTF-8")?;

            if old_content != new_content {
                println!("What you have stored now vs What will be stored if you continue:");
                print_diff(&old_content, &new_content);

                if !args.force
                    && !Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Are you sure you want to change the stored env?")
                        .interact()?
                {
                    return Ok(());
                }
            }
        } else if !args.force {
            // Remote registration without passphrase: just ask for overwrite confirmation
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Environment {}/{} already exists. Overwrite?",
                    args.project, args.env
                ))
                .interact()?
            {
                return Ok(());
            }
        }
    }

    store
        .put(
            args.project,
            args.env,
            &target_filename,
            &encrypted_content_bytes,
        )
        .await?;
    info!(
        "Environment {}/{} registered successfully",
        args.project, args.env
    );

    Ok(())
}

pub async fn set_env(args: SetArgs<'_>, store: &impl Store) -> Result<()> {
    let (target_file_name, encrypted_stored) = match store.get(args.project, args.env).await {
        Ok(data) => data,
        Err(_) => {
            warn!("Env not found");
            return Ok(());
        }
    };

    let target_file_path = std::env::current_dir()?.join(&target_file_name);

    let decrypted_content_bytes = security::decrypt(&encrypted_stored, args.passphrase)
        .context("Failed to decrypt stored env file")?;

    let new_content =
        String::from_utf8(decrypted_content_bytes).context("Stored content is not valid UTF-8")?;

    if target_file_path.exists() {
        let old_content =
            fs::read_to_string(&target_file_path).context("Failed to read existing local file")?;

        if old_content != new_content {
            println!("What you have now vs What you will have if you continue:");
            print_diff(&old_content, &new_content);

            if !args.force
                && !Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Are you sure you want to change the env you have right now?")
                    .interact()?
            {
                return Ok(());
            }
        }
    }

    fs::write(&target_file_path, new_content.as_bytes())
        .context("Failed to write target env file")?;
    info!("{} set to {}", args.env, target_file_name);

    Ok(())
}

pub async fn remove(args: RmArgs<'_>, store: &impl Store) -> Result<()> {
    // Check if exists
    if let Ok((_, encrypted_content)) = store.get(args.project, args.env).await {
        let content_bytes = security::decrypt(&encrypted_content, args.passphrase)
            .context("Failed to decrypt env file for confirmation")?;
        let content = String::from_utf8_lossy(&content_bytes);

        if !args.force {
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Are you sure you want to remove this?:\n\n{}",
                    content
                ))
                .interact()?
            {
                return Ok(());
            }

            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Are you sure? (Final confirmation)")
                .interact()?
            {
                return Ok(());
            }
        }

        store.delete(args.project, args.env).await?;
        info!("Removed env: {}", args.env);
    } else {
        warn!("Env does not exist");
    }

    Ok(())
}

pub async fn list(args: ListArgs<'_>, store: &impl Store) -> Result<()> {
    if let Some(p) = args.project {
        println!("{}", p);
        let envs = store.list_envs(p).await?;
        if envs.is_empty() {
            println!("  (no environments)");
        }
        for env in envs {
            println!("  {}", env);
        }
    } else {
        let projects = store.list_projects().await?;
        if projects.is_empty() {
            println!("No projects found.");
            return Ok(());
        }
        for p in projects {
            println!("{}", p);
            let envs = store.list_envs(&p).await?;
            for env in envs {
                println!("  {}", env);
            }
        }
    }

    Ok(())
}

pub async fn show(args: ShowArgs<'_>, store: &impl Store) -> Result<()> {
    match store.get(args.project, args.env).await {
        Ok((_, encrypted_content)) => {
            let content_bytes = security::decrypt(&encrypted_content, args.passphrase)?;
            let content = String::from_utf8_lossy(&content_bytes);
            println!("{}", content);
        }
        Err(_) => {
            bail!("Environment not found");
        }
    }

    Ok(())
}

pub async fn share(args: ShareArgs<'_>, store: &impl SharableStore) -> Result<()> {
    // Get the hash for this environment
    let hash = store.get_hash(args.project, args.env).await?;

    // Create network node with shared store
    let (ticket, network) = store.start_share_session(&hash).await?;

    info!("Sharing environment: {}/{}", args.project, args.env);
    println!("\nTo clone this environment, run:");
    println!(
        "  envbro register {} {} --ticket {}\n",
        args.project, args.env, ticket
    );
    println!("Press Ctrl+C to stop sharing...");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c()
        .await
        .context("Failed to listen for Ctrl+C")?;

    info!("Shutting down...");
    network.shutdown().await?;

    Ok(())
}

// Helper to show diff
fn print_diff(old: &str, new: &str) {
    let diff = TextDiff::from_lines(old, new);
    for change in diff.iter_all_changes() {
        let (sign, style) = match change.tag() {
            ChangeTag::Delete => ("-", Style::new().red()),
            ChangeTag::Insert => ("+", Style::new().green()),
            ChangeTag::Equal => (" ", Style::new().dim()),
        };
        print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
    }
}
