use crate::network::Network;
use crate::security;
use crate::store::Store;
use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use secrecy::SecretString;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

pub async fn register(
    project: &str,
    env: &str,
    target_path: &str,
    force: bool,
    passphrase: &SecretString,
    store: &impl Store,
) -> Result<()> {
    if env.contains('/') {
        bail!("Environment name cannot contain /");
    }

    let target_path = Path::new(target_path);
    if !target_path.exists() || !target_path.is_file() {
        bail!("Target file doesn't exist or is not a file");
    }

    // Check existing
    if let Ok((_, encrypted_content)) = store.get(project, env).await {
        let old_content_bytes = security::decrypt(&encrypted_content, passphrase)
            .context("Failed to decrypt existing env file")?;
        let old_content =
            String::from_utf8(old_content_bytes).context("Stored content is not valid UTF-8")?;
        let new_content = fs::read_to_string(target_path).context("Failed to read target file")?;

        if old_content != new_content {
            println!("What you have stored now vs What will be stored if you continue:");
            print_diff(&old_content, &new_content);

            if !force
                && !Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Are you sure you want to change the stored env?")
                    .interact()?
            {
                return Ok(());
            }
        }
    }

    let new_content_bytes = fs::read(target_path).context("Failed to read target file")?;
    // Verify it is UTF-8
    let _ = std::str::from_utf8(&new_content_bytes).context("Target file is not valid UTF-8")?;

    let filename = target_path
        .file_name()
        .context("Invalid target filename")?
        .to_string_lossy();

    let encrypted = security::encrypt(&new_content_bytes, passphrase)?;
    store.put(project, env, &filename, &encrypted).await?;
    info!("New env stored");

    Ok(())
}

pub async fn set_env(
    project: &str,
    env: &str,
    force: bool,
    passphrase: &SecretString,
    store: &impl Store,
) -> Result<()> {
    let (target_file_name, encrypted_stored) = match store.get(project, env).await {
        Ok(data) => data,
        Err(_) => {
            warn!("Env not found");
            return Ok(());
        }
    };

    let target_file_path = std::env::current_dir()?.join(&target_file_name);

    let decrypted_content_bytes = security::decrypt(&encrypted_stored, passphrase)
        .context("Failed to decrypt stored env file")?;

    let new_content =
        String::from_utf8(decrypted_content_bytes).context("Stored content is not valid UTF-8")?;

    if target_file_path.exists() {
        let old_content =
            fs::read_to_string(&target_file_path).context("Failed to read existing local file")?;

        if old_content != new_content {
            println!("What you have now vs What you will have if you continue:");
            print_diff(&old_content, &new_content);

            if !force
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
    info!("{} set to {}", env, target_file_name);

    Ok(())
}

pub async fn remove(
    project: &str,
    env: &str,
    force: bool,
    passphrase: &SecretString,
    store: &impl Store,
) -> Result<()> {
    // Check if exists
    if let Ok((_, encrypted_content)) = store.get(project, env).await {
        let content_bytes = security::decrypt(&encrypted_content, passphrase)
            .context("Failed to decrypt env file for confirmation")?;
        let content = String::from_utf8_lossy(&content_bytes);

        if !force {
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

        store.delete(project, env).await?;
        info!("Removed env: {}", env);
    } else {
        warn!("Env does not exist");
    }

    Ok(())
}

pub async fn list(project: Option<&str>, store: &impl Store) -> Result<()> {
    if let Some(p) = project {
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

pub async fn show(
    project: &str,
    env: &str,
    passphrase: &SecretString,
    store: &impl Store,
) -> Result<()> {
    match store.get(project, env).await {
        Ok((_, encrypted_content)) => {
            let content_bytes = security::decrypt(&encrypted_content, passphrase)?;
            let content = String::from_utf8_lossy(&content_bytes);
            println!("{}", content);
        }
        Err(_) => {
            bail!("Environment not found");
        }
    }

    Ok(())
}

pub async fn share(project: &str, env: &str, store: &crate::store::iroh::IrohStore) -> Result<()> {
    use crate::network::IrohNetwork;

    // Get the hash for this environment
    let hash = store.get_hash(project, env).await?;

    // Create network node with shared store
    let network = IrohNetwork::spawn(store.get_store_clone()).await?;

    // Generate ticket
    let ticket = network.share(hash).await?;

    info!("Sharing environment: {}/{}", project, env);
    println!("\nTo clone this environment, run:");
    println!("  envbro clone {}\n", ticket);
    println!("Press Ctrl+C to stop sharing...");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c()
        .await
        .context("Failed to listen for Ctrl+C")?;

    info!("Shutting down...");
    Box::new(network).shutdown().await?;

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
