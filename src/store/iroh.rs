use super::Store;
use anyhow::{Context, Result};
use iroh_blobs::store::fs::FsStore as BlobStore;
use iroh_docs::store::fs::Store as DocStore;
use iroh_docs::Author;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DATA_DIR: &str = "data";
const META_FILE: &str = "iroh.json";
const BLOBS_DIR: &str = "blobs";
const DOCS_FILE: &str = "docs.db";

#[derive(Serialize, Deserialize)]
struct EnvMeta {
    doc_id: iroh_docs::NamespaceId,
    capability: iroh_docs::Capability,
    author: iroh_docs::AuthorId,
    // Envelope Encryption fields
    project_key_enc: Vec<u8>,
    project_key_salt: Vec<u8>,
}

pub struct IrohStore {
    root: PathBuf,
    blob_store: BlobStore,
    doc_store: DocStore,
    author: Author,
}

impl IrohStore {
    pub async fn new() -> Result<Self> {
        let root = get_store_path()?;
        let data_root = root.join(DATA_DIR);
        tokio::fs::create_dir_all(&data_root).await?;

        // Initialize persistent stores
        let blob_path = data_root.join(BLOBS_DIR);
        tokio::fs::create_dir_all(&blob_path).await?;
        let blob_store = BlobStore::load(&blob_path).await?;

        // Note: DocStore::persistent is blocking
        let doc_path = data_root.join(DOCS_FILE);
        let mut doc_store = DocStore::persistent(&doc_path)?;

        // Manage Author
        let mut rng = rand::rng();
        let author = if let Some(existing) = doc_store.list_authors()?.next() {
            existing?
        } else {
            doc_store.new_author(&mut rng)?
        };

        Ok(Self {
            root,
            blob_store,
            doc_store,
            author,
        })
    }

    fn get_meta_path(&self, project: &str, env: &str) -> PathBuf {
        self.root.join(project).join(env).join(META_FILE)
    }

    pub async fn is_initialized(&self, project: &str, env: &str) -> bool {
        tokio::fs::try_exists(self.get_meta_path(project, env))
            .await
            .unwrap_or(false)
    }

    pub async fn init_env(
        &mut self,
        project: &str,
        env: &str,
        project_key_enc: Vec<u8>,
        project_key_salt: Vec<u8>,
    ) -> Result<()> {
        let meta_path = self.get_meta_path(project, env);
        if tokio::fs::try_exists(&meta_path).await? {
            return Err(anyhow::anyhow!("Environment already initialized"));
        }

        // Create new Doc
        let (secret, doc_id, capability, author_id) = {
            let mut rng = rand::rng();
            let secret = iroh_docs::NamespaceSecret::new(&mut rng);
            let id = secret.id();
            let capability = iroh_docs::Capability::Write(secret.clone());
            (secret, id, capability, self.author.id())
        };

        // Initialize and persist the replica in the local database.
        // This registers the document's secret key so it can be reopened later via open_replica.
        let _replica = self.doc_store.new_replica(secret)?;

        let meta = EnvMeta {
            doc_id,
            capability,
            author: author_id,
            project_key_enc,
            project_key_salt,
        };

        if let Some(parent) = meta_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let meta_json = serde_json::to_vec_pretty(&meta)?;
        tokio::fs::write(&meta_path, meta_json).await?;

        Ok(())
    }

    pub async fn get_project_key_meta(
        &self,
        project: &str,
        env: &str,
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        let meta_path = self.get_meta_path(project, env);
        if !tokio::fs::try_exists(&meta_path).await? {
            return Ok(None);
        }

        let meta_content = tokio::fs::read(&meta_path).await?;
        let meta: EnvMeta = serde_json::from_slice(&meta_content)?;

        Ok(Some((meta.project_key_enc, meta.project_key_salt)))
    }

    pub async fn shutdown(self) -> Result<()> {
        self.blob_store.shutdown().await?;
        Ok(())
    }
}

fn get_store_path() -> Result<PathBuf> {
    if let Ok(root) = std::env::var("ENVBRO_ROOT") {
        return Ok(PathBuf::from(root));
    }
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".envbro"))
}

#[async_trait::async_trait]
impl Store for IrohStore {
    async fn set_env_var(
        &mut self,
        project: &str,
        env: &str,
        key: &str,
        value: &[u8],
    ) -> Result<()> {
        let meta_path = self.get_meta_path(project, env);

        // We assume init_env was called before
        let meta_content = tokio::fs::read(&meta_path)
            .await
            .context("Environment not initialized. Run register first.")?;
        let meta: EnvMeta = serde_json::from_slice(&meta_content)?;

        // Open Replica
        let mut replica = self.doc_store.open_replica(&meta.doc_id)?;

        // 1. Store Content in Blob Store
        let tag = self.blob_store.blobs().add_bytes(value.to_vec()).await?;
        let hash = tag.hash;

        // 2. Insert Entry in Doc
        replica.insert(key, &self.author, hash, value.len() as u64)?;

        Ok(())
    }

    async fn set_env_vars(
        &mut self,
        project: &str,
        env: &str,
        entries: Vec<(String, Vec<u8>)>,
    ) -> Result<()> {
        let meta_path = self.get_meta_path(project, env);

        // We assume init_env was called before
        let meta_content = tokio::fs::read(&meta_path)
            .await
            .context("Environment not initialized. Run register first.")?;
        let meta: EnvMeta = serde_json::from_slice(&meta_content)?;

        // Open Replica
        let mut replica = self.doc_store.open_replica(&meta.doc_id)?;

        for (key, value) in entries {
            // 1. Store Content in Blob Store
            let tag = self.blob_store.blobs().add_bytes(value.clone()).await?;
            let hash = tag.hash;

            // 2. Insert Entry in Doc
            replica.insert(key, &self.author, hash, value.len() as u64)?;
        }

        Ok(())
    }

    async fn list_env_vars(&mut self, project: &str, env: &str) -> Result<Vec<(String, Vec<u8>)>> {
        let meta_path = self.get_meta_path(project, env);
        if !tokio::fs::try_exists(&meta_path).await? {
            return Ok(Vec::new());
        }

        let meta_content = tokio::fs::read(&meta_path).await?;
        let meta: EnvMeta = serde_json::from_slice(&meta_content)?;

        // Open Replica
        let replica = self.doc_store.open_replica(&meta.doc_id)?;
        let replica_id = replica.id();
        drop(replica);

        // Query all keys
        use iroh_docs::store::Query;
        let query = Query::single_latest_per_key().build();
        let iter = self.doc_store.get_many(replica_id, query)?;

        let mut results = Vec::new();
        for entry_res in iter {
            let entry = entry_res?;
            let key_bytes = entry.key();
            let key =
                String::from_utf8(key_bytes.to_vec()).unwrap_or_else(|_| hex::encode(key_bytes));

            let hash = entry.content_hash();

            match self.blob_store.blobs().get_bytes(hash).await {
                Ok(bytes) => results.push((key, bytes.to_vec())),
                Err(e) => {
                    eprintln!("Warning: Missing blob for key {}: {}", key, e);
                    return Err(anyhow::anyhow!("Missing blob for key {}: {}", key, e));
                }
            }
        }
        Ok(results)
    }
}
