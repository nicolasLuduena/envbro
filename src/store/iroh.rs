use super::Store;
use anyhow::{Context, Result};
use async_trait::async_trait;
use iroh_blobs::store::fs::Store as FsStore;
use iroh_blobs::store::{Map, MapEntry, Store as _};
use iroh_blobs::Hash;
use iroh_io::AsyncSliceReader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::sync::Mutex;

const MANIFEST_FILE: &str = "manifest.json";
const IROH_DATA_DIR: &str = "iroh-data";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EnvEntry {
    filename: String,
    hash: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct Manifest {
    // Project -> Env -> EnvEntry
    projects: HashMap<String, HashMap<String, EnvEntry>>,
}

pub struct IrohStore {
    store: FsStore,
    manifest_path: PathBuf,
    manifest: Mutex<Manifest>,
}

impl IrohStore {
    pub async fn new() -> Result<Self> {
        let root = get_store_path()?;
        let iroh_data_path = root.join(IROH_DATA_DIR);
        tokio::fs::create_dir_all(&iroh_data_path).await?;

        // Initialize Iroh store (persistent)
        let store = FsStore::load(&iroh_data_path).await?;

        let manifest_path = root.join(MANIFEST_FILE);
        let manifest = if manifest_path.exists() {
            let content = tokio::fs::read_to_string(&manifest_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Manifest::default()
        };

        Ok(Self {
            store,
            manifest_path,
            manifest: Mutex::new(manifest),
        })
    }

    async fn save_manifest(&self) -> Result<()> {
        let manifest = self.manifest.lock().await;
        let content = serde_json::to_string_pretty(&*manifest)?;
        tokio::fs::write(&self.manifest_path, content).await?;
        Ok(())
    }
}

/// Returns the path to the storage directory ~/.envbro
fn get_store_path() -> Result<PathBuf> {
    if let Ok(root) = std::env::var("ENVBRO_ROOT") {
        return Ok(PathBuf::from(root));
    }
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".envbro"))
}

#[async_trait]
impl Store for IrohStore {
    async fn put(&self, project: &str, env: &str, filename: &str, data: &[u8]) -> Result<()> {
        let temp_tag = self
            .store
            .import_bytes(data.to_vec().into(), iroh_blobs::BlobFormat::Raw)
            .await?;
        let hash = *temp_tag.hash();

        // Update manifest
        {
            let mut manifest = self.manifest.lock().await;
            let project_entry = manifest.projects.entry(project.to_string()).or_default();
            project_entry.insert(
                env.to_string(),
                EnvEntry {
                    filename: filename.to_string(),
                    hash: hash.to_string(),
                },
            );
        }

        self.save_manifest().await?;

        Ok(())
    }

    async fn get(&self, project: &str, env: &str) -> Result<(String, Vec<u8>)> {
        let entry_info = {
            let manifest = self.manifest.lock().await;
            manifest
                .projects
                .get(project)
                .and_then(|envs| envs.get(env))
                .cloned()
                .context("Environment not found in manifest")?
        };

        let hash = Hash::from_str(&entry_info.hash)?;
        let entry = self
            .store
            .get(&hash)
            .await?
            .context("Blob not found in store")?;

        // data_reader() returns DataReader directly
        let mut reader = entry.data_reader();

        // AsyncSliceReader has read_at(offset, len) -> Result<Bytes>
        let size = entry.size().value();
        let bytes = reader.read_at(0, size as usize).await?;

        Ok((entry_info.filename, bytes.to_vec()))
    }

    async fn delete(&self, project: &str, env: &str) -> Result<()> {
        // Get the hash before removing from manifest so we can delete the blob
        let hash_to_delete = {
            let manifest = self.manifest.lock().await;
            manifest
                .projects
                .get(project)
                .and_then(|envs| envs.get(env))
                .map(|entry| entry.hash.clone())
        };

        // Delete the blob from Iroh store if it exists
        if let Some(hash_str) = hash_to_delete {
            let hash = Hash::from_str(&hash_str)?;
            // Delete the blob - this removes it from the store
            self.store.delete(vec![hash]).await?;
        }

        // Update manifest
        {
            let mut manifest = self.manifest.lock().await;
            if let Some(envs) = manifest.projects.get_mut(project) {
                envs.remove(env);
                if envs.is_empty() {
                    manifest.projects.remove(project);
                }
            }
        }
        self.save_manifest().await?;
        Ok(())
    }

    async fn list_projects(&self) -> Result<Vec<String>> {
        let manifest = self.manifest.lock().await;
        Ok(manifest.projects.keys().cloned().collect())
    }

    async fn list_envs(&self, project: &str) -> Result<Vec<String>> {
        let manifest = self.manifest.lock().await;
        Ok(manifest
            .projects
            .get(project)
            .map(|envs| envs.keys().cloned().collect())
            .unwrap_or_default())
    }
}
