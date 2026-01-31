use envbro::store::{iroh::IrohStore, Store};
use tempfile::tempdir;

#[tokio::test]
async fn test_iroh_store_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Inject the test path for THIS process.
    // Note: Since this modifies global state, this test should ideally be run in isolation
    // or we should refactor Store to accept a root path in constructor.
    // For now, we assume we are the only unit test modifying this var.
    std::env::set_var("ENVBRO_ROOT", root.to_str().unwrap());

    // 1. Initialize
    let mut store = IrohStore::new().await.expect("Failed to create store");

    // 2. Init Env
    // We mock the encrypted key and salt
    let project_key_enc = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let project_key_salt = vec![0xCA, 0xFE, 0xBA, 0xBE];

    let project = "test_project_batch";
    let env = "dev";

    store
        .init_env(
            project,
            env,
            project_key_enc.clone(),
            project_key_salt.clone(),
        )
        .await
        .expect("Init env failed");

    // 3. Test Batch Set (The new feature!)
    let entries = vec![
        ("KEY1".to_string(), b"VAL1".to_vec()),
        ("KEY2".to_string(), b"VAL2".to_vec()),
        ("KEY3".to_string(), b"VAL3_WITH_SPACES".to_vec()),
    ];

    store
        .set_env_vars(project, env, entries.clone())
        .await
        .expect("Batch set failed");

    // 4. Verify List
    let results = store
        .list_env_vars(project, env)
        .await
        .expect("List failed");

    // We expect 3 items
    assert_eq!(results.len(), 3, "Should have 3 items");

    // Check integrity
    for (key, val) in entries {
        let found = results
            .iter()
            .find(|(k, _)| *k == key)
            .expect("Key missing in stored results");
        assert_eq!(found.1, val, "Value mismatch for key {}", key);
    }

    // 5. Test Update One (Standard Set)
    store
        .set_env_var(project, env, "KEY1", b"NEW_VAL1")
        .await
        .expect("Single set failed");

    let updated_results = store
        .list_env_vars(project, env)
        .await
        .expect("List failed after update");
    let val1 = updated_results.iter().find(|(k, _)| k == "KEY1").unwrap();
    assert_eq!(val1.1, b"NEW_VAL1");
}
