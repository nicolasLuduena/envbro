use crate::security;
use crate::store::{iroh::IrohStore, Store};
use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use secrecy::SecretString;
use similar::{ChangeTag, TextDiff};
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

/// Returns the path to the storage directory ~/.envbro
fn get_store_path() -> Result<PathBuf> {
    if let Ok(root) = std::env::var("ENVBRO_ROOT") {
        return Ok(PathBuf::from(root));
    }
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".envbro"))
}

pub async fn register(
    project: &str,
    env: &str,
    target_path: &str,
    force: bool,
    passphrase: &SecretString,
) -> Result<()> {
    if env.contains('/') {
        bail!("Environment name cannot contain /");
    }

    let target_path = Path::new(target_path);
    if !tokio::fs::try_exists(target_path).await? {
        bail!("Target path doesn't exist");
    }
    if !tokio::fs::metadata(target_path).await?.is_file() {
        bail!("Target path must be a file");
    }

    let mut store = IrohStore::new().await?;
    let is_update;
    let project_key;

    // 1. Initialization / Key Retrieval
    if store.is_initialized(project, env).await {
        // Retrieve and Unwrap Project Key
        let (key_enc, key_salt) = store
            .get_project_key_meta(project, env)
            .await?
            .context("Failed to get project key meta")?;

        project_key = security::decrypt_project_key(&key_enc, &key_salt, passphrase)
            .context("Failed to decrypt project key (passphrase mismatch?)")?;
        is_update = true;
    } else {
        // Initialize New
        project_key = security::generate_project_key();
        let (key_enc, key_salt) = security::encrypt_project_key(&project_key, passphrase)?;

        store.init_env(project, env, key_enc, key_salt).await?;
        info!("Initialized new environment {}/{}", project, env);
        is_update = false;
    }

    // Read target file content once for comparison and storing
    let new_env_content = tokio::fs::read(target_path).await?;

    // 2. Check existing variables (Differential Update)
    if is_update {
        let current_vars = store.list_env_vars(project, env).await?;
        let mut old_content = String::new();

        if !current_vars.is_empty() {
            let mut sorted_vars = current_vars;
            sorted_vars.sort_by(|a, b| a.0.cmp(&b.0));

            for (key, val_enc) in sorted_vars {
                let val_dec = security::decrypt(&val_enc, &project_key)
                    .context("Failed to decrypt existing variable (corrupted?)")?;
                let val_str =
                    String::from_utf8(val_dec).context("Invalid UTF-8 in stored value")?;
                old_content.push_str(&format!("{}={}\n", key, val_str));
            }

            let new_vars_iter = dotenvy::from_read_iter(std::io::Cursor::new(&new_env_content));
            let mut new_content_clean = String::new();
            for item in new_vars_iter {
                let (k, v) = item?;
                new_content_clean.push_str(&format!("{}={}\n", k, v));
            }

            if old_content != new_content_clean {
                println!(
                    "Changes detected (Note: Comments and formatting are not preserved in storage):"
                );
                print_diff(&old_content, &new_content_clean);

                if !force
                    && !Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Are you sure you want to update the stored environment?")
                        .interact()?
                {
                    return Ok(());
                }
            }
        }
    }

    // 3. Write new variables
    let new_vars_iter = dotenvy::from_read_iter(std::io::Cursor::new(&new_env_content));
    let mut batch_entries = Vec::new();

    for item in new_vars_iter {
        let (key, value) = item?;
        // Fast Encryption using Project Key
        let encrypted = security::encrypt(value.as_bytes(), &project_key)?;
        batch_entries.push((key, encrypted));
    }

    if !batch_entries.is_empty() {
        store.set_env_vars(project, env, batch_entries).await?;
    }

    info!("New env stored in Iroh Doc");
    store.shutdown().await?;

    Ok(())
}

pub async fn set_env(
    project: &str,
    env: &str,
    force: bool,
    passphrase: &SecretString,
) -> Result<()> {
    let mut store = IrohStore::new().await?;

    // 1. Get Key
    if !store.is_initialized(project, env).await {
        warn!("Env does not exist");
        return Ok(());
    }

    let (key_enc, key_salt) = store
        .get_project_key_meta(project, env)
        .await?
        .context("Failed to get project key meta")?;

    let project_key = security::decrypt_project_key(&key_enc, &key_salt, passphrase)
        .context("Failed to decrypt project key (passphrase mismatch?)")?;

    // 2. List variables
    let current_vars = store.list_env_vars(project, env).await?;
    if current_vars.is_empty() {
        warn!("Env is empty");
        return Ok(());
    }

    let mut sorted_vars = current_vars;
    sorted_vars.sort_by(|a, b| a.0.cmp(&b.0));

    let mut new_content = String::new();
    for (key, val_enc) in sorted_vars {
        let val_dec = security::decrypt(&val_enc, &project_key)?;
        let val_str = String::from_utf8(val_dec)?;
        new_content.push_str(&format!("{}={}\n", key, val_str));
    }

    // Target file: .env in current directory?
    let target_filename = ".env";
    let target_file_path = std::env::current_dir()?.join(target_filename);

    if tokio::fs::try_exists(&target_file_path).await? {
        let old_content = tokio::fs::read_to_string(&target_file_path).await?;
        if old_content != new_content {
            println!("Local .env differs from stored env:");
            print_diff(&old_content, &new_content);

            if !force
                && !Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Overwrite local .env?")
                    .interact()?
            {
                return Ok(());
            }
        }
    }

    tokio::fs::write(&target_file_path, new_content).await?;
    info!("{} set to .env", env);

    Ok(())
}

pub async fn remove(
    project: &str,
    env: &str,
    force: bool,
    passphrase: &SecretString,
) -> Result<()> {
    // Current implementation removes directory.
    let store_root = get_store_path()?;
    let env_path = store_root.join(project).join(env);

    let mut store = IrohStore::new().await?;

    if !store.is_initialized(project, env).await {
        warn!("Env does not exist");
        return Ok(());
    }

    // Verify Passphrase (Authentication)
    // We shouldn't allow deleting an environment if we can't unlock it.
    let (key_enc, key_salt) = store
        .get_project_key_meta(project, env)
        .await?
        .context("Failed to read environment metadata")?;

    // This will error if the passphrase is incorrect
    security::decrypt_project_key(&key_enc, &key_salt, passphrase)
        .context("Authentication failed: Invalid passphrase")?;

    if !force {
        let vars = store.list_env_vars(project, env).await?;
        let count = vars.len();

        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Are you sure you want to remove env '{}' ({} variables)?",
                env, count
            ))
            .interact()?
        {
            return Ok(());
        }
    }

    tokio::fs::remove_dir_all(&env_path)
        .await
        .context("Failed to remove env directory")?;
    info!("Removed env: {}", env);

    // Cleanup project dir if empty
    let project_path = store_root.join(project);
    if let Ok(mut entries) = tokio::fs::read_dir(&project_path).await {
        if entries.next_entry().await.ok().flatten().is_none() {
            tokio::fs::remove_dir(&project_path).await.ok();
        }
    }

    Ok(())
}

pub async fn list(project: Option<&str>) -> Result<()> {
    let store_root = get_store_path()?;
    let search_root = if let Some(p) = project {
        store_root.join(p)
    } else {
        store_root
    };

    if !tokio::fs::try_exists(&search_root).await? {
        println!("No environments found.");
        return Ok(());
    }

    println!("{}", search_root.display());

    for entry in WalkDir::new(&search_root)
        .min_depth(1)
        .max_depth(3)
        .sort_by_file_name()
    {
        let entry = entry?;
        let depth = entry.depth();
        let indent = "  ".repeat(depth);
        let name = entry.file_name().to_string_lossy();
        println!("{}{}", indent, name);
    }

    Ok(())
}

pub async fn show(project: &str, env: &str, passphrase: &SecretString) -> Result<()> {
    let mut store = IrohStore::new().await?;

    if !store.is_initialized(project, env).await {
        bail!("Environment does not exist");
    }

    // 1. Get Key
    let (key_enc, key_salt) = store
        .get_project_key_meta(project, env)
        .await?
        .context("Failed to get project key meta")?;

    let project_key = security::decrypt_project_key(&key_enc, &key_salt, passphrase)
        .context("Failed to decrypt project key (passphrase mismatch?)")?;

    // 2. Encrypt/Decrypt
    let vars = store.list_env_vars(project, env).await?;

    if vars.is_empty() {
        bail!("Environment is empty");
    }

    let mut sorted_vars = vars;
    sorted_vars.sort_by(|a, b| a.0.cmp(&b.0));

    for (key, val_enc) in sorted_vars {
        let val_dec = security::decrypt(&val_enc, &project_key)?;
        let val_str = String::from_utf8_lossy(&val_dec);
        println!("{}={}", key, val_str);
    }

    Ok(())
}

// Helper query
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
