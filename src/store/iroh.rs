use super::{SharableStore, Store};
use crate::network::{IrohNetwork, Network};
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
use tracing::warn;

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

impl Manifest {
    fn is_hash_used(&self, hash: &str) -> bool {
        self.projects
            .values()
            .flat_map(|envs| envs.values())
            .any(|e| e.hash == hash)
    }

    fn remove_entry(&mut self, project: &str, env: &str) {
        if let Some(envs) = self.projects.get_mut(project) {
            envs.remove(env);
            if envs.is_empty() {
                self.projects.remove(project);
            }
        }
    }
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

        // Atomic write strategy:
        // 1. Write to a temporary file
        // 2. Backup existing file (if any)
        // 3. Rename temporary file to target file

        let tmp_path = self.manifest_path.with_extension("tmp");
        let backup_path = self.manifest_path.with_extension("bak");

        // 1. Write to temp file
        tokio::fs::write(&tmp_path, &content)
            .await
            .context("Failed to write manifest to temp file")?;

        // 2. Backup existing (ignore error if it doesn't exist, but log others)
        if self.manifest_path.exists() {
            if let Err(e) = tokio::fs::copy(&self.manifest_path, &backup_path).await {
                warn!("Failed to create manifest backup: {}", e);
            }
        }

        // 3. Atomic rename (POSIX atomic, Windows atomic-ish)
        tokio::fs::rename(&tmp_path, &self.manifest_path)
            .await
            .context("Failed to rename temp manifest to target")?;
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
        let new_hash = *temp_tag.hash();

        // Check for orphan cleanup and update manifest
        let hash_to_prune = {
            let mut manifest = self.manifest.lock().await;

            // 1. Get existing hash (if any)
            let old_hash = manifest
                .projects
                .get(project)
                .and_then(|envs| envs.get(env))
                .map(|e| e.hash.clone());

            // 2. Update Manifest with new entry
            let project_entry = manifest.projects.entry(project.to_string()).or_default();
            project_entry.insert(
                env.to_string(),
                EnvEntry {
                    filename: filename.to_string(),
                    hash: new_hash.to_string(),
                },
            );

            // 3. Determine if old blob is orphaned
            old_hash.filter(|old_h| {
                // Only consider pruning if hash changed
                old_h != &new_hash.to_string() && !manifest.is_hash_used(old_h)
            })
        };

        // 4. Persist manifest FIRST (safety)
        self.save_manifest().await?;

        // 5. Prune orphan if identified
        if let Some(h_str) = hash_to_prune {
            if let Ok(hash) = Hash::from_str(&h_str) {
                // If this fails, it's not critical (just a leaked blob)
                if let Err(e) = self.store.delete(vec![hash]).await {
                    warn!("Failed to prune orphaned blob {}: {}", h_str, e);
                }
            }
        }

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
        let hash_to_delete = {
            let mut manifest = self.manifest.lock().await;

            // 1. Get the hash of the entry we are about to remove
            let hash = manifest
                .projects
                .get(project)
                .and_then(|envs| envs.get(env))
                .map(|entry| entry.hash.clone());

            // 2. Remove the entry from the manifest (in memory)
            if hash.is_some() {
                manifest.remove_entry(project, env);
            }

            // 3. Check if orphaned (filter returns the hash only if !is_used)
            hash.filter(|h| !manifest.is_hash_used(h))
        };

        // 4. Save the manifest FIRST (so we don't end up with a pointing-to-deleted state)
        self.save_manifest().await?;

        // 5. Delete the blob if it was unique to the removed entry
        if let Some(hash_str) = hash_to_delete {
            let hash = Hash::from_str(&hash_str)?;
            self.store.delete(vec![hash]).await?;
        }

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

    async fn get_hash(&self, project: &str, env: &str) -> Result<String> {
        let manifest = self.manifest.lock().await;
        manifest
            .projects
            .get(project)
            .and_then(|envs| envs.get(env))
            .map(|entry| entry.hash.clone())
            .context("Environment not found in manifest")
    }
}

#[async_trait]
impl SharableStore for IrohStore {
    async fn start_share_session(
        &self,
        hash: &str,
    ) -> Result<(String, Box<dyn crate::network::Network>)> {
        // Create network node with shared store
        // We clone the store handle, which is cheap (Arc internally)
        let network = IrohNetwork::spawn(self.store.clone()).await?;
        let hash = Hash::from_str(hash)?;

        // Generate ticket
        let ticket = network.share(hash).await?;
        Ok((ticket, Box::new(network)))
    }
}

impl IrohStore {
    // get_hash implementation removed in favor of trait implementation
    // get_store_clone removed as it is no longer needed publicly
}
