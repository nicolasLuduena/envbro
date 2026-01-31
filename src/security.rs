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

#[derive(Clone, Zeroize, Debug, PartialEq)]
#[zeroize(drop)]
pub struct ProjectKey(pub [u8; 32]);

/// Generates a random 32-byte project key.
pub fn generate_project_key() -> ProjectKey {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    ProjectKey(key)
}

/// Derives a 32-byte master key from a passphrase and salt using Argon2.
fn derive_master_key(passphrase: &SecretString, salt: &[u8]) -> Result<[u8; 32]> {
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

/// Encrypts the Project Key using the Passphrase (Envelope Layer).
/// Returns (EncryptedKey, Salt).
pub fn encrypt_project_key(
    project_key: &ProjectKey,
    passphrase: &SecretString,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let salt = generate_salt();
    let mut master_key = derive_master_key(passphrase, &salt)?;

    let cipher = Aes256Gcm::new(&master_key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, project_key.0.as_slice())
        .map_err(|e| anyhow::anyhow!("Key wrap failure: {}", e))?;

    // Format: NONCE (12) | CIPHERTEXT
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);

    master_key.zeroize();

    Ok((result, salt.to_vec()))
}

/// Decrypts the Project Key using the Passphrase.
pub fn decrypt_project_key(
    encrypted_key: &[u8],
    salt: &[u8],
    passphrase: &SecretString,
) -> Result<ProjectKey> {
    if encrypted_key.len() < 12 {
        return Err(anyhow::anyhow!("Encrypted key too short"));
    }

    let mut master_key = derive_master_key(passphrase, salt)?;
    let cipher = Aes256Gcm::new(&master_key.into());

    let (nonce_bytes, ciphertext) = encrypted_key.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Key unwrap failed (invalid passphrase)"))?;

    master_key.zeroize();

    if plaintext.len() != 32 {
        return Err(anyhow::anyhow!("Invalid decrypted key length"));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&plaintext);
    Ok(ProjectKey(key))
}

/// Encrypts data using the Project Key (Fast Layer).
/// No Argon2 here, just straight AES-GCM.
/// Returns [NONCE (12) | CIPHERTEXT + TAG]
pub fn encrypt(data: &[u8], key: &ProjectKey) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(&key.0.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failure: {}", e))?;

    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypts data using the Project Key (Fast Layer).
pub fn decrypt(data: &[u8], key: &ProjectKey) -> Result<Vec<u8>> {
    if data.len() < 12 {
        return Err(anyhow::anyhow!("Data too short"));
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(&key.0.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed"))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_flow() {
        let passphrase = SecretString::from("soy batman");
        let project_key = generate_project_key();

        // 1. Wrap Key
        let (enc_key, salt) =
            encrypt_project_key(&project_key, &passphrase).expect("Key wrap failed");

        // 2. Encrypt Data with Project Key
        let data = b"secret payload";
        let enc_data = encrypt(data, &project_key).expect("Data encrypt failed");

        // 3. Unwrap Key
        let unwrapped_key =
            decrypt_project_key(&enc_key, &salt, &passphrase).expect("Key unwrap failed");
        assert_eq!(project_key, unwrapped_key);

        // 4. Decrypt Data
        let dec_data = decrypt(&enc_data, &unwrapped_key).expect("Data decrypt failed");
        assert_eq!(data, dec_data.as_slice());
    }

    #[test]
    fn test_wrong_passphrase_envelope() {
        let passphrase = SecretString::from("soy batman");
        let project_key = generate_project_key();
        let (enc_key, salt) = encrypt_project_key(&project_key, &passphrase).unwrap();

        let wrong_pass = SecretString::from("joker");
        let result = decrypt_project_key(&enc_key, &salt, &wrong_pass);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_empty_data() {
        let key = generate_project_key();
        let data = b"";
        let enc = encrypt(data, &key).expect("Encryption failed");
        let dec = decrypt(&enc, &key).expect("Decryption failed");
        assert_eq!(dec, data);
    }
}
