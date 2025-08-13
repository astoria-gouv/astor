//! Data encryption and decryption utilities

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::errors::AstorError;

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub data: String,           // Base64 encoded encrypted data
    pub nonce: String,          // Base64 encoded nonce
    pub key_id: String,         // Key identifier used for encryption
    pub algorithm: String,      // Encryption algorithm used
    pub created_at: DateTime<Utc>,
}

impl EncryptedData {
    /// Create new encrypted data container
    pub fn new(
        encrypted_bytes: Vec<u8>,
        nonce: Vec<u8>,
        key_id: String,
        algorithm: String,
    ) -> Self {
        Self {
            data: general_purpose::STANDARD.encode(encrypted_bytes),
            nonce: general_purpose::STANDARD.encode(nonce),
            key_id,
            algorithm,
            created_at: Utc::now(),
        }
    }

    /// Get encrypted data as bytes
    pub fn get_encrypted_bytes(&self) -> Result<Vec<u8>, AstorError> {
        general_purpose::STANDARD
            .decode(&self.data)
            .map_err(|e| AstorError::CryptographicError(format!("Base64 decode error: {}", e)))
    }

    /// Get nonce as bytes
    pub fn get_nonce_bytes(&self) -> Result<Vec<u8>, AstorError> {
        general_purpose::STANDARD
            .decode(&self.nonce)
            .map_err(|e| AstorError::CryptographicError(format!("Nonce decode error: {}", e)))
    }
}

/// Encryption key metadata
#[derive(Debug, Clone)]
struct EncryptionKey {
    id: String,
    key: Vec<u8>,
    created_at: DateTime<Utc>,
    algorithm: String,
    is_active: bool,
}

impl EncryptionKey {
    fn new(algorithm: String) -> Self {
        let key = match algorithm.as_str() {
            "AES-256-GCM" => Aes256Gcm::generate_key(OsRng).to_vec(),
            _ => panic!("Unsupported algorithm"),
        };

        Self {
            id: Uuid::new_v4().to_string(),
            key,
            created_at: Utc::now(),
            algorithm,
            is_active: true,
        }
    }

    fn should_rotate(&self) -> bool {
        let rotation_period = chrono::Duration::days(90);
        Utc::now() - self.created_at > rotation_period
    }
}

/// Encryption manager for handling data encryption/decryption
pub struct EncryptionManager {
    keys: HashMap<String, EncryptionKey>,
    active_key_id: String,
    master_key: Vec<u8>,
}

impl EncryptionManager {
    /// Create new encryption manager with master key
    pub fn new(master_key_str: &str) -> Result<Self, AstorError> {
        // Derive master key from string using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(master_key_str.as_bytes());
        let master_key = hasher.finalize().to_vec();

        // Generate initial encryption key
        let initial_key = EncryptionKey::new("AES-256-GCM".to_string());
        let active_key_id = initial_key.id.clone();
        
        let mut keys = HashMap::new();
        keys.insert(active_key_id.clone(), initial_key);

        Ok(Self {
            keys,
            active_key_id,
            master_key,
        })
    }

    /// Encrypt data using active key
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData, AstorError> {
        let active_key = self.keys
            .get(&self.active_key_id)
            .ok_or(AstorError::CryptographicError("Active key not found".to_string()))?;

        match active_key.algorithm.as_str() {
            "AES-256-GCM" => self.encrypt_aes_gcm(plaintext, active_key),
            _ => Err(AstorError::CryptographicError("Unsupported algorithm".to_string())),
        }
    }

    /// Decrypt data using specified key
    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>, AstorError> {
        let key = self.keys
            .get(&encrypted_data.key_id)
            .ok_or(AstorError::CryptographicError("Decryption key not found".to_string()))?;

        match encrypted_data.algorithm.as_str() {
            "AES-256-GCM" => self.decrypt_aes_gcm(encrypted_data, key),
            _ => Err(AstorError::CryptographicError("Unsupported algorithm".to_string())),
        }
    }

    /// Encrypt string data
    pub fn encrypt_string(&self, plaintext: &str) -> Result<EncryptedData, AstorError> {
        self.encrypt(plaintext.as_bytes())
    }

    /// Decrypt to string
    pub fn decrypt_string(&self, encrypted_data: &EncryptedData) -> Result<String, AstorError> {
        let decrypted_bytes = self.decrypt(encrypted_data)?;
        String::from_utf8(decrypted_bytes)
            .map_err(|e| AstorError::CryptographicError(format!("UTF-8 decode error: {}", e)))
    }

    /// Rotate encryption keys
    pub fn rotate_keys(&mut self) -> Result<(), AstorError> {
        // Check if current key needs rotation
        let current_key = self.keys.get(&self.active_key_id).unwrap();
        if !current_key.should_rotate() {
            return Ok(());
        }

        // Generate new key
        let new_key = EncryptionKey::new("AES-256-GCM".to_string());
        let new_key_id = new_key.id.clone();

        // Mark old key as inactive
        if let Some(old_key) = self.keys.get_mut(&self.active_key_id) {
            old_key.is_active = false;
        }

        // Add new key and set as active
        self.keys.insert(new_key_id.clone(), new_key);
        self.active_key_id = new_key_id;

        Ok(())
    }

    /// Get encryption statistics
    pub fn get_encryption_stats(&self) -> EncryptionStats {
        let total_keys = self.keys.len();
        let active_keys = self.keys.values().filter(|k| k.is_active).count();
        let keys_needing_rotation = self.keys.values().filter(|k| k.should_rotate()).count();

        EncryptionStats {
            total_keys,
            active_keys,
            keys_needing_rotation,
            active_key_id: self.active_key_id.clone(),
        }
    }

    /// Clean up old inactive keys (keep for 1 year for decryption)
    pub fn cleanup_old_keys(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::days(365);
        let keys_to_remove: Vec<String> = self.keys
            .iter()
            .filter(|(_, key)| !key.is_active && key.created_at < cutoff)
            .map(|(id, _)| id.clone())
            .collect();

        for key_id in keys_to_remove {
            self.keys.remove(&key_id);
        }
    }

    /// AES-256-GCM encryption implementation
    fn encrypt_aes_gcm(&self, plaintext: &[u8], key: &EncryptionKey) -> Result<EncryptedData, AstorError> {
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key.key);
        let cipher = Aes256Gcm::new(cipher_key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| AstorError::CryptographicError(format!("AES encryption error: {}", e)))?;

        Ok(EncryptedData::new(
            ciphertext,
            nonce.to_vec(),
            key.id.clone(),
            "AES-256-GCM".to_string(),
        ))
    }

    /// AES-256-GCM decryption implementation
    fn decrypt_aes_gcm(&self, encrypted_data: &EncryptedData, key: &EncryptionKey) -> Result<Vec<u8>, AstorError> {
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key.key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        let ciphertext = encrypted_data.get_encrypted_bytes()?;
        let nonce_bytes = encrypted_data.get_nonce_bytes()?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| AstorError::CryptographicError(format!("AES decryption error: {}", e)))
    }
}

/// Encryption statistics
#[derive(Debug, Serialize)]
pub struct EncryptionStats {
    pub total_keys: usize,
    pub active_keys: usize,
    pub keys_needing_rotation: usize,
    pub active_key_id: String,
}

/// Utility functions for common encryption tasks
impl EncryptionManager {
    /// Encrypt sensitive configuration data
    pub fn encrypt_config(&self, config_data: &serde_json::Value) -> Result<EncryptedData, AstorError> {
        let json_string = serde_json::to_string(config_data)
            .map_err(|e| AstorError::CryptographicError(format!("JSON serialization error: {}", e)))?;
        self.encrypt_string(&json_string)
    }

    /// Decrypt configuration data
    pub fn decrypt_config(&self, encrypted_data: &EncryptedData) -> Result<serde_json::Value, AstorError> {
        let json_string = self.decrypt_string(encrypted_data)?;
        serde_json::from_str(&json_string)
            .map_err(|e| AstorError::CryptographicError(format!("JSON deserialization error: {}", e)))
    }

    /// Encrypt database connection strings
    pub fn encrypt_connection_string(&self, connection_string: &str) -> Result<EncryptedData, AstorError> {
        self.encrypt_string(connection_string)
    }

    /// Decrypt database connection strings
    pub fn decrypt_connection_string(&self, encrypted_data: &EncryptedData) -> Result<String, AstorError> {
        self.decrypt_string(encrypted_data)
    }

    /// Encrypt API keys and secrets
    pub fn encrypt_secret(&self, secret: &str) -> Result<EncryptedData, AstorError> {
        self.encrypt_string(secret)
    }

    /// Decrypt API keys and secrets
    pub fn decrypt_secret(&self, encrypted_data: &EncryptedData) -> Result<String, AstorError> {
        self.decrypt_string(encrypted_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let manager = EncryptionManager::new("test_master_key").unwrap();
        let plaintext = "Hello, World!";
        
        let encrypted = manager.encrypt_string(plaintext).unwrap();
        let decrypted = manager.decrypt_string(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_key_rotation() {
        let mut manager = EncryptionManager::new("test_master_key").unwrap();
        let original_key_id = manager.active_key_id.clone();
        
        // Force key rotation by modifying the key creation time
        if let Some(key) = manager.keys.get_mut(&original_key_id) {
            key.created_at = Utc::now() - chrono::Duration::days(91);
        }
        
        manager.rotate_keys().unwrap();
        assert_ne!(original_key_id, manager.active_key_id);
    }

    #[test]
    fn test_config_encryption() {
        let manager = EncryptionManager::new("test_master_key").unwrap();
        let config = serde_json::json!({
            "database_url": "postgresql://user:pass@localhost/db",
            "api_key": "secret_key_123"
        });
        
        let encrypted = manager.encrypt_config(&config).unwrap();
        let decrypted = manager.decrypt_config(&encrypted).unwrap();
        
        assert_eq!(config, decrypted);
    }
}
