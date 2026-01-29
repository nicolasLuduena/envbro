use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use similar::{ChangeTag, TextDiff};
use std::fs;
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

pub fn register(project: &str, env: &str, target_path: &str, force: bool) -> Result<()> {
    if env.contains('/') {
        bail!("Environment name cannot contain /");
    }

    let target_path = Path::new(target_path);
    if !target_path.exists() || !target_path.is_file() {
        bail!("Target file doesn't exist or is not a file");
    }

    let store_root = get_store_path()?;
    let env_path = store_root.join(project).join(env);

    // Create directory
    fs::create_dir_all(&env_path).context("Failed to create env directory")?;

    let file_name = target_path.file_name().context("Invalid target filename")?;
    let stored_env_path = env_path.join(file_name);

    // Check existing
    if stored_env_path.exists() {
        let old_content =
            fs::read_to_string(&stored_env_path).context("Failed to read existing stored env")?;
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

    fs::copy(target_path, stored_env_path).context("Failed to copy env file")?;
    info!("New env stored");

    Ok(())
}

pub fn set_env(project: &str, env: &str, force: bool) -> Result<()> {
    let store_root = get_store_path()?;
    let env_path = store_root.join(project).join(env);

    if !env_path.exists() || !env_path.is_dir() {
        warn!("Env does not exist");
        return Ok(());
    }

    // Find the first file in the directory (assuming one env file per env folder)
    let mut entries = fs::read_dir(&env_path)?;
    let entry = match entries.next() {
        Some(Ok(e)) => e,
        _ => {
            warn!("Env directory is empty");
            return Ok(());
        }
    };

    let stored_file_path = entry.path();
    let file_name = stored_file_path
        .file_name()
        .context("Invalid stored filename")?;
    let target_file_path = std::env::current_dir()?.join(file_name);

    if target_file_path.exists() {
        let old_content =
            fs::read_to_string(&target_file_path).context("Failed to read existing local file")?;
        let new_content =
            fs::read_to_string(&stored_file_path).context("Failed to read stored env file")?;

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

    fs::copy(&stored_file_path, &target_file_path)
        .context("Failed to copy env file to current directory")?;
    info!("{} set", env);

    Ok(())
}

pub fn remove(project: &str, env: &str, force: bool) -> Result<()> {
    let store_root = get_store_path()?;
    let env_path = store_root.join(project).join(env);

    if !env_path.exists() || !env_path.is_dir() {
        warn!("Env does not exist");
        return Ok(());
    }

    let mut entries = fs::read_dir(&env_path)?;
    if let Some(Ok(entry)) = entries.next() {
        let content = fs::read_to_string(entry.path()).context("Failed to read env file")?;

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
    }

    fs::remove_dir_all(&env_path).context("Failed to remove env directory")?;
    info!("Removed env: {}", env);

    // Cleanup project dir if empty
    let project_path = store_root.join(project);
    if let Ok(entries) = fs::read_dir(&project_path) {
        if entries.count() == 0 {
            fs::remove_dir(&project_path).ok(); // Ignore if fails
        }
    }

    Ok(())
}

pub fn list(project: Option<&str>) -> Result<()> {
    let store_root = get_store_path()?;
    let search_root = if let Some(p) = project {
        store_root.join(p)
    } else {
        store_root
    };

    if !search_root.exists() {
        println!("No environments found.");
        return Ok(());
    }

    // Simple tree-like view
    println!("{}", search_root.display());
    let _prefix_root = search_root.clone();

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

pub fn show(project: &str, env: &str) -> Result<()> {
    let store_root = get_store_path()?;
    let env_path = store_root.join(project).join(env);

    if !env_path.exists() || !env_path.is_dir() {
        bail!("Project environment does not exist");
    }

    let mut entries = fs::read_dir(&env_path)?;
    match entries.next() {
        Some(Ok(entry)) => {
            let content = fs::read_to_string(entry.path()).context("Failed to read env file")?;
            println!("{}", content);
        }
        _ => {
            bail!("Environment directory is empty");
        }
    }

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
