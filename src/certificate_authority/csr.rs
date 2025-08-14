//! Certificate Signing Request implementation

use ed25519_dalek::PublicKey;
use serde::{Deserialize, Serialize};

use super::certificate::CertificateSubject;
use crate::errors::AstorError;
use crate::security::{KeyPair, Signature};

/// Certificate Signing Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSigningRequest {
    pub version: u8,
    pub subject: CertificateSubject,
    pub public_key: Vec<u8>,
    pub attributes: CsrAttributes,
    pub subject_alternative_names: Vec<String>,
    pub signature_algorithm: String,
    pub signature: Vec<u8>,
}

impl CertificateSigningRequest {
    /// Create new CSR
    pub fn new(
        subject: CertificateSubject,
        keypair: &KeyPair,
        attributes: CsrAttributes,
        subject_alternative_names: Vec<String>,
    ) -> Result<Self, AstorError> {
        let public_key = keypair.public_key().as_bytes().to_vec();

        let mut csr = Self {
            version: 1,
            subject,
            public_key,
            attributes,
            subject_alternative_names,
            signature_algorithm: "Ed25519".to_string(),
            signature: vec![],
        };

        // Sign CSR
        let signature = csr.sign_csr(keypair)?;
        csr.signature = signature.to_base64().into_bytes();

        Ok(csr)
    }

    /// Sign CSR with private key
    fn sign_csr(&self, keypair: &KeyPair) -> Result<Signature, AstorError> {
        let tbs_data = self.to_be_signed_bytes()?;
        Ok(keypair.sign(&tbs_data))
    }

    /// Get CSR data to be signed
    fn to_be_signed_bytes(&self) -> Result<Vec<u8>, AstorError> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.version.to_be_bytes());
        data.extend_from_slice(serde_json::to_string(&self.subject)?.as_bytes());
        data.extend_from_slice(&self.public_key);
        data.extend_from_slice(serde_json::to_string(&self.attributes)?.as_bytes());
        Ok(data)
    }

    /// Verify CSR signature
    pub fn verify_signature(&self) -> Result<bool, AstorError> {
        let public_key = PublicKey::from_bytes(&self.public_key)
            .map_err(|e| AstorError::CryptographicError(e.to_string()))?;

        let tbs_data = self.to_be_signed_bytes()?;
        let signature = Signature::from_base64(
            &String::from_utf8(self.signature.clone())?,
            "csr_signature".to_string(),
        )?;

        match signature.verify(&public_key, &tbs_data) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Export CSR as PEM format
    pub fn to_pem(&self) -> Result<String, AstorError> {
        let csr_data = serde_json::to_vec(self)?;
        let encoded = base64::encode(csr_data);

        Ok(format!(
            "-----BEGIN CERTIFICATE REQUEST-----\n{}\n-----END CERTIFICATE REQUEST-----",
            encoded
        ))
    }
}

/// CSR attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrAttributes {
    pub challenge_password: Option<String>,
    pub unstructured_name: Option<String>,
    pub requested_extensions: Vec<String>,
}

/// CSR processor for validation and handling
pub struct CsrProcessor {
    validation_rules: CsrValidationRules,
}

impl CsrProcessor {
    pub fn new() -> Self {
        Self {
            validation_rules: CsrValidationRules::default(),
        }
    }

    /// Validate CSR before processing
    pub fn validate_csr(&self, csr: &CertificateSigningRequest) -> Result<(), AstorError> {
        // Verify signature
        if !csr.verify_signature()? {
            return Err(AstorError::InvalidSignature);
        }

        // Validate subject fields
        self.validate_subject(&csr.subject)?;

        // Validate public key
        self.validate_public_key(&csr.public_key)?;

        // Additional validation rules
        self.apply_validation_rules(csr)?;

        Ok(())
    }

    fn validate_subject(&self, subject: &CertificateSubject) -> Result<(), AstorError> {
        if subject.common_name.is_empty() {
            return Err(AstorError::ValidationError(
                "Common name is required".to_string(),
            ));
        }

        if subject.organization.is_empty() {
            return Err(AstorError::ValidationError(
                "Organization is required".to_string(),
            ));
        }

        if subject.country.len() != 2 {
            return Err(AstorError::ValidationError(
                "Country must be 2-letter code".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_public_key(&self, public_key: &[u8]) -> Result<(), AstorError> {
        if public_key.len() != 32 {
            return Err(AstorError::ValidationError(
                "Invalid public key length".to_string(),
            ));
        }

        // Verify it's a valid Ed25519 public key
        PublicKey::from_bytes(public_key)
            .map_err(|_| AstorError::ValidationError("Invalid Ed25519 public key".to_string()))?;

        Ok(())
    }

    fn apply_validation_rules(&self, csr: &CertificateSigningRequest) -> Result<(), AstorError> {
        // Apply custom validation rules
        for rule in &self.validation_rules.rules {
            rule.validate(csr)?;
        }
        Ok(())
    }
}

/// CSR validation rules
#[derive(Debug, Clone)]
pub struct CsrValidationRules {
    pub rules: Vec<Box<dyn CsrValidationRule>>,
}

impl Default for CsrValidationRules {
    fn default() -> Self {
        Self { rules: vec![] }
    }
}

/// Trait for CSR validation rules
pub trait CsrValidationRule: Send + Sync {
    fn validate(&self, csr: &CertificateSigningRequest) -> Result<(), AstorError>;
}
