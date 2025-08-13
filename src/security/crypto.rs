//! Enhanced cryptographic operations

use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{rand_core::RngCore, SaltString}};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use base64::{Engine as _, engine::general_purpose};

use crate::errors::AstorError;

/// Enhanced wrapper for Ed25519 key pair with additional security
#[derive(Clone)]
pub struct KeyPair {
    keypair: Keypair,
    created_at: chrono::DateTime<chrono::Utc>,
    key_id: String,
}

impl KeyPair {
    /// Generate a new random key pair with metadata
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        let key_id = uuid::Uuid::new_v4().to_string();
        
        Self { 
            keypair,
            created_at: chrono::Utc::now(),
            key_id,
        }
    }

    /// Create from existing secret key bytes with validation
    pub fn from_bytes(secret_bytes: &[u8]) -> Result<Self, AstorError> {
        if secret_bytes.len() != 32 {
            return Err(AstorError::CryptographicError("Invalid key length".to_string()));
        }

        let secret_key = SecretKey::from_bytes(secret_bytes)
            .map_err(|e| AstorError::CryptographicError(e.to_string()))?;
        let public_key = PublicKey::from(&secret_key);
        let keypair = Keypair { secret: secret_key, public: public_key };
        let key_id = uuid::Uuid::new_v4().to_string();
        
        Ok(Self { 
            keypair,
            created_at: chrono::Utc::now(),
            key_id,
        })
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        self.keypair.public
    }

    /// Get key ID for tracking
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Sign a message with additional metadata
    pub fn sign(&self, message: &[u8]) -> Signature {
        let signature = self.keypair.sign(message);
        Signature { 
            signature,
            key_id: self.key_id.clone(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Export public key as base64
    pub fn public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.keypair.public.as_bytes())
    }

    /// Check if key should be rotated (older than 90 days)
    pub fn should_rotate(&self) -> bool {
        let ninety_days = chrono::Duration::days(90);
        chrono::Utc::now() - self.created_at > ninety_days
    }
}

/// Enhanced digital signature with metadata
#[derive(Clone, Serialize, Deserialize)]
pub struct Signature {
    signature: ed25519_dalek::Signature,
    key_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl Signature {
    /// Verify signature with enhanced validation
    pub fn verify(&self, public_key: &PublicKey, message: &[u8]) -> Result<(), AstorError> {
        // Check signature age (reject signatures older than 5 minutes)
        let max_age = chrono::Duration::minutes(5);
        if chrono::Utc::now() - self.timestamp > max_age {
            return Err(AstorError::InvalidSignature);
        }

        public_key
            .verify(message, &self.signature)
            .map_err(|_| AstorError::InvalidSignature)
    }

    /// Get signature as base64
    pub fn to_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.signature.to_bytes())
    }

    /// Create from base64 string
    pub fn from_base64(data: &str, key_id: String) -> Result<Self, AstorError> {
        let bytes = general_purpose::STANDARD.decode(data)
            .map_err(|_| AstorError::CryptographicError("Invalid base64".to_string()))?;
        
        let signature = ed25519_dalek::Signature::from_bytes(&bytes)
            .map_err(|_| AstorError::CryptographicError("Invalid signature bytes".to_string()))?;

        Ok(Self {
            signature,
            key_id,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Secure password hashing using Argon2
pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl PasswordHasher {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    /// Hash password with salt
    pub fn hash_password(&self, password: &str) -> Result<String, AstorError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AstorError::CryptographicError(e.to_string()))?;
        
        Ok(password_hash.to_string())
    }

    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AstorError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AstorError::CryptographicError(e.to_string()))?;
        
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Enhanced hash functions with different algorithms
pub fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn hash_data_sha512(data: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Secure random number generation
pub fn generate_secure_random(length: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; length];
    OsRng.fill_bytes(&mut buffer);
    buffer
}

/// Generate cryptographically secure API key
pub fn generate_api_key() -> String {
    let random_bytes = generate_secure_random(32);
    general_purpose::STANDARD.encode(random_bytes)
}

/// Time-based one-time password (TOTP) for MFA
pub struct TotpGenerator {
    secret: Vec<u8>,
}

impl TotpGenerator {
    pub fn new() -> Self {
        let secret = generate_secure_random(32);
        Self { secret }
    }

    pub fn generate_code(&self, timestamp: Option<u64>) -> String {
        let time = timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() / 30
        });

        let time_bytes = time.to_be_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&self.secret);
        hasher.update(&time_bytes);
        let hash = hasher.finalize();
        
        let offset = (hash[hash.len() - 1] & 0xf) as usize;
        let code = u32::from_be_bytes([
            hash[offset] & 0x7f,
            hash[offset + 1],
            hash[offset + 2],
            hash[offset + 3],
        ]) % 1_000_000;

        format!("{:06}", code)
    }

    pub fn verify_code(&self, code: &str, window: u32) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() / 30;

        for i in 0..=window {
            let test_time = current_time.saturating_sub(i as u64);
            if self.generate_code(Some(test_time)) == code {
                return true;
            }
            if i > 0 {
                let test_time = current_time + i as u64;
                if self.generate_code(Some(test_time)) == code {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_secret_base32(&self) -> String {
        general_purpose::STANDARD.encode(&self.secret)
    }
}
