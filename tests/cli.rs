use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_register_and_list() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().to_str().unwrap();

    // Create a dummy env file
    let src_env = temp_dir.path().join(".env.test");
    fs::write(&src_env, "FOO=bar").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_envbro"));
    cmd.env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "testpass")
        .arg("register")
        .arg("myproject")
        .arg("prod")
        .arg("--path")
        .arg(src_env.to_str().unwrap())
        .assert()
        .success()
        .stderr(predicate::str::contains("New env stored"));

    // Verify it exists in the store
    // With Iroh, we check for the metadata file
    let stored_meta = temp_dir.path().join("myproject/prod/iroh.json");
    assert!(stored_meta.exists());

    // Test List
    let mut cmd_list = Command::new(env!("CARGO_BIN_EXE_envbro"));
    cmd_list
        .env("ENVBRO_ROOT", root)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("myproject"))
        .stdout(predicate::str::contains("prod"));
}

#[test]
fn test_set() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().to_str().unwrap();
    let cwd = tempdir().unwrap();

    // Setup store with one env using register
    let env_src = temp_dir.path().join("source.env");
    fs::write(&env_src, "SECRET=123").unwrap();

    Command::new(env!("CARGO_BIN_EXE_envbro"))
        .env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "testpass")
        .arg("register")
        .arg("myproject")
        .arg("dev")
        .arg("--path")
        .arg(env_src.to_str().unwrap())
        .assert()
        .success();

    // Run set command
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_envbro"));
    cmd.current_dir(cwd.path())
        .env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "testpass")
        .arg("set")
        .arg("myproject")
        .arg("dev")
        .assert()
        .success()
        .stderr(predicate::str::contains("dev set"));

    // Verify file copied to CWD and is decrypted
    // The command always writes to ".env"
    let target = cwd.path().join(".env");
    assert!(target.exists());
    assert_eq!(fs::read_to_string(target).unwrap(), "SECRET=123\n");
}

#[test]
fn test_show() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().to_str().unwrap();

    // Setup store with register
    let env_src = temp_dir.path().join("source.env");
    fs::write(&env_src, "KEY=VALUE").unwrap();

    Command::new(env!("CARGO_BIN_EXE_envbro"))
        .env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "testpass")
        .arg("register")
        .arg("myproject")
        .arg("staging")
        .arg("--path")
        .arg(env_src.to_str().unwrap())
        .assert()
        .success();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_envbro"));
    cmd.env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "testpass")
        .arg("show")
        .arg("myproject")
        .arg("staging")
        .assert()
        .success()
        .stdout(predicate::str::contains("KEY=VALUE"));
}

#[test]
fn test_remove() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().to_str().unwrap();

    // Setup store with register
    let env_src = temp_dir.path().join("source.env");
    fs::write(&env_src, "DELETE_ME=1").unwrap();

    Command::new(env!("CARGO_BIN_EXE_envbro"))
        .env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "testpass")
        .arg("register")
        .arg("myproject")
        .arg("test")
        .arg("--path")
        .arg(env_src.to_str().unwrap())
        .assert()
        .success();

    // Verify it exists
    assert!(temp_dir.path().join("myproject/test").exists());

    // Run remove with --force
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_envbro"));
    cmd.env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "testpass")
        .arg("rm")
        .arg("myproject")
        .arg("test")
        .arg("--force")
        .assert()
        .success()
        .stderr(predicate::str::contains("Removed env: test"));

    // Verify removed
    assert!(!temp_dir.path().join("myproject/test").exists());
    // Also verify project dir removed since it was empty
    assert!(!temp_dir.path().join("myproject").exists());
}

#[test]
fn test_register_update_needs_force() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().to_str().unwrap();
    let env_src = temp_dir.path().join("dup.env");
    fs::write(&env_src, "A=B").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_envbro"));
    cmd.env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "pass")
        .arg("register")
        .arg("dup_proj")
        .arg("dev")
        .arg("--path")
        .arg(env_src.to_str().unwrap())
        .assert()
        .success();

    // Change content to trigger diff
    fs::write(&env_src, "A=C").unwrap();

    // Try again without force - should try to prompt and fail (no tty)
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_envbro"));
    cmd2.env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "pass")
        .arg("register")
        .arg("dup_proj")
        .arg("dev")
        .arg("--path")
        .arg(env_src.to_str().unwrap())
        .assert()
        .failure();
}

#[test]
fn test_wrong_passphrase_on_show() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().to_str().unwrap();
    let env_src = temp_dir.path().join("secret.env");
    fs::write(&env_src, "SECRET=Batman").unwrap();

    // Register with "correct"
    Command::new(env!("CARGO_BIN_EXE_envbro"))
        .env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "correct")
        .arg("register")
        .arg("sec_proj")
        .arg("prod")
        .arg("--path")
        .arg(env_src.to_str().unwrap())
        .assert()
        .success();

    // Show with "wrong"
    Command::new(env!("CARGO_BIN_EXE_envbro"))
        .env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "wrong")
        .arg("show")
        .arg("sec_proj")
        .arg("prod")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Key unwrap failed"));
}

#[test]
fn test_missing_source_file() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().to_str().unwrap();

    Command::new(env!("CARGO_BIN_EXE_envbro"))
        .env("ENVBRO_ROOT", root)
        .env("ENVBRO_PASSPHRASE", "pass")
        .arg("register")
        .arg("missing_proj")
        .arg("dev")
        .arg("--path")
        .arg("/this/path/does/not/exist/hopefully")
        .assert()
        .failure();
}
