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

    let mut cmd = Command::cargo_bin("envbro").unwrap();
    cmd.env("ENVBRO_ROOT", root)
        .arg("register")
        .arg("myproject")
        .arg("prod")
        .arg("--path")
        .arg(src_env.to_str().unwrap())
        .assert()
        .success()
        .stderr(predicate::str::contains("New env stored"));

    // Verify it exists in the store
    let stored_file = temp_dir.path().join("myproject/prod/.env.test");
    assert!(stored_file.exists());
    assert_eq!(fs::read_to_string(stored_file).unwrap(), "FOO=bar");

    // Test List
    let mut cmd_list = Command::cargo_bin("envbro").unwrap();
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

    // Setup store with one env
    let env_dir = temp_dir.path().join("myproject/dev");
    fs::create_dir_all(&env_dir).unwrap();
    fs::write(env_dir.join(".env.dev"), "SECRET=123").unwrap();

    // Run set command
    let mut cmd = Command::cargo_bin("envbro").unwrap();
    cmd.current_dir(cwd.path())
        .env("ENVBRO_ROOT", root)
        .arg("set")
        .arg("myproject")
        .arg("dev")
        .assert()
        .success()
        .stderr(predicate::str::contains("dev set"));

    // Verify file copied to CWD
    let target = cwd.path().join(".env.dev");
    assert!(target.exists());
    assert_eq!(fs::read_to_string(target).unwrap(), "SECRET=123");
}

#[test]
fn test_show() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().to_str().unwrap();

    // Setup store
    let env_dir = temp_dir.path().join("myproject/staging");
    fs::create_dir_all(&env_dir).unwrap();
    fs::write(env_dir.join(".env.staging"), "KEY=VALUE").unwrap();

    let mut cmd = Command::cargo_bin("envbro").unwrap();
    cmd.env("ENVBRO_ROOT", root)
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

    // Setup store
    let env_dir = temp_dir.path().join("myproject/test");
    fs::create_dir_all(&env_dir).unwrap();
    fs::write(env_dir.join(".env.test"), "DELETE_ME=1").unwrap();

    // Run remove with --force
    let mut cmd = Command::cargo_bin("envbro").unwrap();
    cmd.env("ENVBRO_ROOT", root)
        .arg("rm")
        .arg("myproject")
        .arg("test")
        .arg("--force")
        .assert()
        .success()
        .stderr(predicate::str::contains("Removed env: test"));

    // Verify removed
    assert!(!env_dir.exists());
    // Also verify project dir removed since it was empty
    assert!(!temp_dir.path().join("myproject").exists());
}
