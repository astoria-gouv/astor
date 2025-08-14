//! Security module for cryptographic operations and access control

use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::errors::AstorError;

/// Wrapper for Ed25519 key pair
#[derive(Clone)]
pub struct KeyPair {
    keypair: Keypair,
}

impl KeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        Self { keypair }
    }

    /// Create from existing secret key bytes
    pub fn from_bytes(secret_bytes: &[u8]) -> Result<Self, AstorError> {
        let secret_key = SecretKey::from_bytes(secret_bytes)
            .map_err(|e| AstorError::CryptographicError(e.to_string()))?;
        let public_key = PublicKey::from(&secret_key);
        let keypair = Keypair { secret: secret_key, public: public_key };
        Ok(Self { keypair })
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        self.keypair.public
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        let signature = self.keypair.sign(message);
        Signature { signature }
    }
}

/// Digital signature wrapper
#[derive(Clone, Serialize, Deserialize)]
pub struct Signature {
    signature: ed25519_dalek::Signature,
}

impl Signature {
    /// Verify signature against public key and message
    pub fn verify(&self, public_key: &PublicKey, message: &[u8]) -> Result<(), AstorError> {
        public_key
            .verify(message, &self.signature)
            .map_err(|_| AstorError::InvalidSignature)
    }
}

/// Role-based access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    RootAdmin,
    BankAdmin,
    CentralBankAdmin,
}

impl Role {
    /// Check if role can issue currency
    pub fn can_issue_currency(&self) -> bool {
        matches!(self, Role::RootAdmin | Role::CentralBankAdmin | Role::BankAdmin)
    }

    /// Check if role can manage other admins
    pub fn can_manage_admins(&self) -> bool {
        matches!(self, Role::RootAdmin | Role::CentralBankAdmin)
    }
}

/// Hash function for data integrity
pub fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
