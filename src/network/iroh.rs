//! IrohNetwork implementation for Iroh P2P connectivity.

use super::Network;
use anyhow::{Context, Result};
use async_trait::async_trait;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::downloader::DownloadRequest;
use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::store::fs::Store as FsStore;
use iroh_blobs::store::{Map, MapEntry, Store};
use iroh_blobs::ticket::BlobTicket;
use iroh_blobs::{BlobFormat, Hash, HashAndFormat};
use iroh_io::AsyncSliceReader;

/// Manages P2P networking using Iroh.
pub struct IrohNetwork {
    router: Router,
    blobs: Blobs<FsStore>,
}

impl IrohNetwork {
    /// Create and spawn a new Iroh node with the given blob store.
    pub async fn spawn(store: FsStore) -> Result<Self> {
        // TODO: add possibility of using owner's infra
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let blobs = Blobs::builder(store).build(&endpoint);
        let router = Router::builder(endpoint)
            .accept(iroh_blobs::ALPN, blobs.clone())
            .spawn();

        Ok(Self { router, blobs })
    }

    /// Get a reference to the blobs protocol handler.
    pub fn blobs(&self) -> &Blobs<FsStore> {
        &self.blobs
    }

    /// Get a reference to the underlying store.
    pub fn store(&self) -> &FsStore {
        self.blobs.store()
    }
}

#[async_trait]
impl Network for IrohNetwork {
    async fn share(&self, hash: Hash) -> Result<String> {
        let node_addr = self.router.endpoint().node_addr().await?;
        let ticket = BlobTicket::new(node_addr, hash, BlobFormat::Raw)?;
        Ok(ticket.to_string())
    }

    async fn connect_and_download(&self, ticket_str: &str) -> Result<(Hash, Vec<u8>)> {
        let ticket: BlobTicket = ticket_str.parse()?;
        let hash = ticket.hash();
        let format = ticket.format();
        let node_addr = ticket.node_addr().clone();

        // Create download request with HashAndFormat
        let hash_and_format = HashAndFormat::new(hash, format);
        let request = DownloadRequest::new(hash_and_format, vec![node_addr]);

        // Queue the download and await completion
        self.blobs
            .downloader()
            .queue(request)
            .await
            .await
            .context("Download failed")?;

        // Read the downloaded content from the store
        let entry = self
            .blobs
            .store()
            .get(&hash)
            .await?
            .context("Blob not found after download")?;

        let mut reader = entry.data_reader();
        let size = entry.size().value();
        let bytes = reader.read_at(0, size as usize).await?;

        Ok((hash, bytes.to_vec()))
    }

    async fn shutdown(self: Box<Self>) -> Result<()> {
        self.router.shutdown().await?;
        Ok(())
    }
}
