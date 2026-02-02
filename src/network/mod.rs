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
