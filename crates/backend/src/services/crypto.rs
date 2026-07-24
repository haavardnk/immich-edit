use std::fs;
use std::io;
use std::path::Path;

use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const KEY_VERSION: i64 = 1;
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn to_utf8(&self) -> Option<String> {
        std::str::from_utf8(&self.0).ok().map(str::to_owned)
    }
}

impl std::fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretBytes(***)")
    }
}

#[derive(Clone)]
pub struct Encrypted {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub key_version: i64,
}

impl std::fmt::Debug for Encrypted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Encrypted")
            .field("key_version", &self.key_version)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("instance key missing but encrypted data exists; refusing to start")]
    KeyMissing,
    #[error("instance key is corrupt")]
    KeyCorrupt,
    #[error("unsupported key version {0}")]
    KeyVersion(i64),
    #[error("decrypt failed")]
    Decrypt,
}

pub struct InstanceCrypto {
    cipher: Aes256Gcm,
}

impl std::fmt::Debug for InstanceCrypto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InstanceCrypto")
    }
}

impl InstanceCrypto {
    pub fn load_or_create(path: &Path, has_secrets: bool) -> Result<Self, CryptoError> {
        let key_bytes = match fs::read(path) {
            Ok(bytes) => {
                if bytes.len() != KEY_LEN {
                    return Err(CryptoError::KeyCorrupt);
                }
                bytes
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if has_secrets {
                    return Err(CryptoError::KeyMissing);
                }
                Self::generate_key_file(path)?
            }
            Err(e) => return Err(CryptoError::Io(e)),
        };
        let mut key_arr = [0u8; KEY_LEN];
        key_arr.copy_from_slice(&key_bytes);
        let key = Key::<Aes256Gcm>::from(key_arr);
        Ok(Self {
            cipher: Aes256Gcm::new(&key),
        })
    }

    fn generate_key_file(path: &Path) -> Result<Vec<u8>, CryptoError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let key = Aes256Gcm::generate_key(OsRng);
        let bytes = key.to_vec();
        write_secret_file(path, &bytes)?;
        Ok(bytes)
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Encrypted, CryptoError> {
        let nonce = Aes256Gcm::generate_nonce(OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| CryptoError::Decrypt)?;
        Ok(Encrypted {
            ciphertext,
            nonce: nonce.to_vec(),
            key_version: KEY_VERSION,
        })
    }

    pub fn decrypt(&self, enc: &Encrypted) -> Result<SecretBytes, CryptoError> {
        if enc.key_version != KEY_VERSION {
            return Err(CryptoError::KeyVersion(enc.key_version));
        }
        if enc.nonce.len() != NONCE_LEN {
            return Err(CryptoError::Decrypt);
        }
        let mut nonce_bytes = [0u8; NONCE_LEN];
        nonce_bytes.copy_from_slice(&enc.nonce);
        let nonce = aes_gcm::aead::Nonce::<Aes256Gcm>::from(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(&nonce, enc.ciphertext.as_slice())
            .map_err(|_| CryptoError::Decrypt)?;
        Ok(SecretBytes::new(plaintext))
    }
}

#[cfg(unix)]
fn write_secret_file(path: &Path, bytes: &[u8]) -> Result<(), CryptoError> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn write_secret_file(path: &Path, bytes: &[u8]) -> Result<(), CryptoError> {
    fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_recovers_plaintext() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("instance.key");
        let crypto = InstanceCrypto::load_or_create(&path, false).unwrap();
        let enc = crypto.encrypt(b"immich-token-secret").unwrap();
        if enc.ciphertext == b"immich-token-secret" {
            panic!("ciphertext must not equal plaintext");
        }
        let dec = crypto.decrypt(&enc).unwrap();
        assert_eq!(dec.as_slice(), b"immich-token-secret");
    }

    #[test]
    fn ciphertext_differs_per_encryption() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("instance.key");
        let crypto = InstanceCrypto::load_or_create(&path, false).unwrap();
        let a = crypto.encrypt(b"same").unwrap();
        let b = crypto.encrypt(b"same").unwrap();
        assert_ne!(a.nonce, b.nonce);
        assert_ne!(a.ciphertext, b.ciphertext);
    }

    #[cfg(unix)]
    #[test]
    fn key_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("instance.key");
        InstanceCrypto::load_or_create(&path, false).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn persisted_key_decrypts_across_loads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("instance.key");
        let enc = InstanceCrypto::load_or_create(&path, false)
            .unwrap()
            .encrypt(b"persist")
            .unwrap();
        let reopened = InstanceCrypto::load_or_create(&path, true).unwrap();
        assert_eq!(reopened.decrypt(&enc).unwrap().as_slice(), b"persist");
    }

    #[test]
    fn missing_key_with_secrets_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("instance.key");
        let err = InstanceCrypto::load_or_create(&path, true).unwrap_err();
        assert!(matches!(err, CryptoError::KeyMissing));
    }

    #[test]
    fn wrong_key_version_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("instance.key");
        let crypto = InstanceCrypto::load_or_create(&path, false).unwrap();
        let mut enc = crypto.encrypt(b"x").unwrap();
        enc.key_version = 999;
        assert!(matches!(
            crypto.decrypt(&enc),
            Err(CryptoError::KeyVersion(999))
        ));
    }

    #[test]
    fn secret_debug_is_redacted() {
        let s = SecretBytes::new(b"topsecret".to_vec());
        assert_eq!(format!("{s:?}"), "SecretBytes(***)");
    }
}
