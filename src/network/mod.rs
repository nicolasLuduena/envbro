//! # Network Module
//!
//! Provides a generic `Network` trait and implementations for P2P networking.

pub mod iroh;

use ::iroh_blobs::Hash;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Network: Send + Sync {
    /// Share a blob with value `hash`. Returns a ticket string.
    async fn share(&self, hash: Hash) -> Result<String>;

    /// Connect to a peer using `ticket` and download the blob.
    /// Returns the hash and the content bytes.
    async fn connect_and_download(&self, ticket: &str) -> Result<(Hash, Vec<u8>)>;

    /// Gracefully shutdown the network node.
    async fn shutdown(self: Box<Self>) -> Result<()>;
}

pub use iroh::IrohNetwork;

/// Helper to download content from a ticket using an ephemeral node.
/// Returns the hash (as string) and the content bytes.
pub async fn fetch_from_ticket(ticket: &str) -> Result<(String, Vec<u8>)> {
    use anyhow::Context;
    use iroh_blobs::store::fs::Store as FsStore;
    use tempfile::TempDir;
    use tracing::{info, warn};

    info!("Connecting to peer and downloading environment...");

    // Create a temporary store for the download operation
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let temp_store = FsStore::load(temp_dir.path())
        .await
        .context("Failed to create temporary store")?;

    // Initialize network with temporary store
    let network = IrohNetwork::spawn(temp_store).await?;
    let result = network.connect_and_download(ticket).await;

    let shutdown_result = Box::new(network).shutdown().await;

    let (hash, bytes) = result?;

    if let Err(e) = shutdown_result {
        warn!("Network shutdown error (non-fatal): {:#}", e);
    }

    Ok((hash.to_string(), bytes))
}
