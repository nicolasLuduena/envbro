//! P2P Sync Integration Test
//!
//! Tests the full provider -> downloader flow using IrohNetwork.

use anyhow::Result;
use envbro::network::{IrohNetwork, Network};
use iroh_blobs::store::fs::Store as FsStore;
use iroh_blobs::store::Store as IrohStoreTrait;
use iroh_blobs::BlobFormat;
use tempfile::TempDir;

#[tokio::test]
async fn test_p2p_blob_transfer() -> Result<()> {
    // Create temporary directories for provider and downloader
    let provider_dir = TempDir::new()?;
    let downloader_dir = TempDir::new()?;

    // Initialize stores
    let provider_store = FsStore::load(provider_dir.path()).await?;
    let downloader_store = FsStore::load(downloader_dir.path()).await?;

    // Secret data to share
    let secret_data = b"im'actuallybatman";

    // Provider: add data using Store trait's import_bytes
    let tag = provider_store
        .import_bytes(secret_data.to_vec().into(), BlobFormat::Raw)
        .await?;
    let hash = *tag.hash();

    // Spawn provider node
    let provider = IrohNetwork::spawn(provider_store).await?;
    let ticket_str = provider.share(hash).await?;

    println!("Provider sharing ticket: {}", ticket_str);

    // Downloader: use ticket string to download
    // Box the downloader to test the trait object behavior or just use it directly
    // The trait uses Box<Self> for shutdown, so we keep ownership until then.
    let downloader = IrohNetwork::spawn(downloader_store).await?;
    let (downloaded_hash, downloaded_bytes) = downloader.connect_and_download(&ticket_str).await?;

    // Verify
    assert_eq!(downloaded_hash, hash);
    assert_eq!(downloaded_bytes, secret_data);

    // Cleanup
    Box::new(provider).shutdown().await?;
    Box::new(downloader).shutdown().await?;

    Ok(())
}
