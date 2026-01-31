use anyhow::Result;
use iroh_blobs::store::mem::MemStore;
use iroh_docs::store::fs::Store as DocStore;
use rand::rng;

#[tokio::test]
async fn test_iroh_local_ops() -> Result<()> {
    // 1. Setup Stores
    // Blob Store (for content)
    let blob_store = MemStore::new();

    // Doc Store (for keys/metadata)
    let mut doc_store = DocStore::memory();

    // 2. Create Identity
    let mut rng = rng();
    let author = doc_store.new_author(&mut rng)?;
    let namespace = iroh_docs::NamespaceSecret::new(&mut rng);
    let mut replica = doc_store.new_replica(namespace)?;
    println!("Created Doc: {}", replica.id());

    // 3. Store Content (Value)
    let key = b"API_KEY";
    let value = b"encrypted_ciphertext_simulation";

    // Add to Blob Store
    let tag = blob_store.blobs().add_bytes(value.to_vec()).await?;
    let hash = tag.hash;
    println!("Stored Blob Hash: {}", hash);

    // 4. Store Entry (Key -> Hash)
    replica.insert(key, &author, hash, value.len() as u64)?;
    println!("Inserted Key into Doc");

    let replica_id = replica.id();
    drop(replica);

    // 5. Read Back
    // Get latest entry for key
    let exact = doc_store.get_exact(replica_id, author.id(), key, false)?;
    let entry = exact.expect("Should find entry");

    assert_eq!(entry.content_hash(), hash);

    // Get content from Blob Store
    let content_bytes = blob_store.blobs().get_bytes(hash).await?;
    assert_eq!(content_bytes.as_ref(), value);

    println!(
        "Successfully read back value: {:?}",
        String::from_utf8_lossy(&content_bytes)
    );

    Ok(())
}
