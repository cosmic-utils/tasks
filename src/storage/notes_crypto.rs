//! Optional at-rest encryption for the `Task::notes` field.
//!
//! Encrypted payloads are written as `enc:v1:<base64(nonce|ciphertext)>` so
//! plaintext stays plaintext (the prefix is the tell) and reads can
//! auto-detect without a flag. The key is a 32-byte secret stored in the
//! system keyring under (`dev.edfloreshz.Tasks.notes`, `master`); it is
//! generated on first use and never leaves the device.
//!
//! CalDAV roundtrips deliberately see *plaintext*: the storage layer
//! decrypts on read and re-encrypts on write, so the sync engine pushes
//! the readable form to remote calendars (preserving interop with other
//! clients that share the same calendar).

use base64::Engine as _;
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;

const SERVICE: &str = "dev.edfloreshz.Tasks.notes";
const ACCOUNT: &str = "master";
pub const PREFIX: &str = "enc:v1:";

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("keyring: {0}")]
    Keyring(String),
    #[error("key has wrong length")]
    KeyLength,
    #[error("base64: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("ciphertext too short")]
    Truncated,
    #[error("aead: {0}")]
    Aead(String),
    #[error("not utf-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

fn entry() -> Result<keyring::Entry, CryptoError> {
    keyring::Entry::new(SERVICE, ACCOUNT).map_err(|e| CryptoError::Keyring(e.to_string()))
}

fn load_or_create_key() -> Result<[u8; 32], CryptoError> {
    let entry = entry()?;
    match entry.get_password() {
        Ok(s) => {
            let bytes = base64::engine::general_purpose::STANDARD.decode(s.trim())?;
            if bytes.len() != 32 {
                return Err(CryptoError::KeyLength);
            }
            let mut k = [0u8; 32];
            k.copy_from_slice(&bytes);
            Ok(k)
        }
        Err(keyring::Error::NoEntry) => {
            let mut k = [0u8; 32];
            rand::rng().fill_bytes(&mut k);
            entry
                .set_password(&base64::engine::general_purpose::STANDARD.encode(k))
                .map_err(|e| CryptoError::Keyring(e.to_string()))?;
            Ok(k)
        }
        Err(e) => Err(CryptoError::Keyring(e.to_string())),
    }
}

/// True when `s` looks like a payload produced by [`encrypt`].
pub fn is_encrypted(s: &str) -> bool {
    s.starts_with(PREFIX)
}

/// Encrypt `plain`. Empty strings and already-encrypted strings are
/// returned untouched so callers can apply this idempotently.
pub fn encrypt(plain: &str) -> Result<String, CryptoError> {
    if plain.is_empty() || is_encrypted(plain) {
        return Ok(plain.to_string());
    }
    let key = load_or_create_key()?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plain.as_bytes())
        .map_err(|e| CryptoError::Aead(e.to_string()))?;
    let mut combined = Vec::with_capacity(12 + ct.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ct);
    Ok(format!(
        "{PREFIX}{}",
        base64::engine::general_purpose::STANDARD.encode(combined)
    ))
}

/// Decrypt `text` if it looks like an encrypted payload; otherwise return
/// it unchanged. This makes reads tolerant of mixed-format storage during
/// the migration window after toggling the feature on.
pub fn decrypt(text: &str) -> Result<String, CryptoError> {
    if !is_encrypted(text) {
        return Ok(text.to_string());
    }
    let body = &text[PREFIX.len()..];
    let bytes = base64::engine::general_purpose::STANDARD.decode(body.trim())?;
    if bytes.len() < 12 {
        return Err(CryptoError::Truncated);
    }
    let key = load_or_create_key()?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let nonce = Nonce::from_slice(&bytes[..12]);
    let pt = cipher
        .decrypt(nonce, &bytes[12..])
        .map_err(|e| CryptoError::Aead(e.to_string()))?;
    Ok(String::from_utf8(pt)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_passes_through() {
        assert_eq!(encrypt("").unwrap(), "");
        assert_eq!(decrypt("").unwrap(), "");
    }

    #[test]
    fn plaintext_decrypt_is_identity() {
        assert_eq!(decrypt("hello").unwrap(), "hello");
    }

    #[test]
    fn is_encrypted_recognizes_prefix() {
        assert!(is_encrypted("enc:v1:abc"));
        assert!(!is_encrypted("plain notes"));
    }
}
