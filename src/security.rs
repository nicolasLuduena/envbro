use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::RngCore, PasswordHasher, SaltString},
    Argon2,
};
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

pub type Key = [u8; 32];

/// Derives a 32-byte key from a passphrase and salt using Argon2.
pub fn derive_key(passphrase: &SecretString, salt: &[u8]) -> Result<Key> {
    let mut key = [0u8; 32];
    let argon2 = Argon2::default();
    let salt_string =
        SaltString::encode_b64(salt).map_err(|e| anyhow::anyhow!("Invalid salt: {}", e))?;

    let password_hash = argon2
        .hash_password(passphrase.expose_secret().as_bytes(), &salt_string)
        .map_err(|e| anyhow::anyhow!("Argon2 error: {}", e))?;

    let hash = password_hash
        .hash
        .ok_or_else(|| anyhow::anyhow!("No hash output"))?;

    if hash.len() != 32 {
        return Err(anyhow::anyhow!("Derived key length mismatch"));
    }

    key.copy_from_slice(hash.as_bytes());
    Ok(key)
}

/// Generate a random 16-byte salt for Argon2
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Encrypts data using AES-256-GCM.
/// Returns a vector containing [SALT (16) | NONCE (12) | CIPHERTEXT + TAG].
pub fn encrypt(data: &[u8], passphrase: &SecretString) -> Result<Vec<u8>> {
    let salt = generate_salt();
    let mut key = derive_key(passphrase, &salt)?;

    let cipher = Aes256Gcm::new(&key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failure: {}", e))?;

    let mut result = Vec::with_capacity(salt.len() + nonce.len() + ciphertext.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);

    // Zeroize key from memory
    key.zeroize();

    Ok(result)
}

/// Decrypts data using AES-256-GCM.
/// Expects data format: [SALT (16) | NONCE (12) | CIPHERTEXT + TAG]
pub fn decrypt(data: &[u8], passphrase: &SecretString) -> Result<Vec<u8>> {
    if data.len() < 16 + 12 {
        return Err(anyhow::anyhow!("Data too short to contain salt and nonce"));
    }

    let (salt, rest) = data.split_at(16);
    let (nonce_bytes, ciphertext) = rest.split_at(12);

    let mut key = derive_key(passphrase, salt)?;
    let cipher = Aes256Gcm::new(&key.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed (invalid passphrase or corrupted data)"))?;

    // Zeroize key from memory
    key.zeroize();

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let passphrase = SecretString::from("soy batman");
        let data = b"secret data";

        let encrypted = encrypt(data, &passphrase).expect("Encryption failed");
        assert_ne!(data, encrypted.as_slice());

        let decrypted = decrypt(&encrypted, &passphrase).expect("Decryption failed");
        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_wrong_passphrase() {
        let passphrase = SecretString::from("i'm batman");
        let data = b"secret data";

        let encrypted = encrypt(data, &passphrase).expect("Encryption failed");

        let result = decrypt(&encrypted, &SecretString::from("wrongpassword"));
        assert!(result.is_err());
    }
}
