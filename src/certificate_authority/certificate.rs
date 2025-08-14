//! Certificate implementation for Astor Currency PKI

use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::PublicKey;
use serde::{Deserialize, Serialize};

use super::csr::CertificateSigningRequest;
use crate::errors::AstorError;
use crate::security::{KeyPair, Signature};

/// Digital certificate for Astor Currency operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    version: u8,
    serial_number: String,
    issuer: CertificateSubject,
    subject: CertificateSubject,
    public_key: Vec<u8>, // Ed25519 public key bytes
    not_before: DateTime<Utc>,
    not_after: DateTime<Utc>,
    certificate_type: CertificateType,
    extensions: CertificateExtensions,
    signature_algorithm: String,
    signature: Vec<u8>,
    status: CertificateStatus,
}

impl Certificate {
    /// Create new root CA certificate (self-signed)
    pub fn new_root_ca(
        public_key: PublicKey,
        organization: String,
        country: String,
        validity_years: u32,
    ) -> Result<Self, AstorError> {
        let now = Utc::now();
        let not_after = now + Duration::days(validity_years as i64 * 365);

        let subject = CertificateSubject {
            common_name: format!("{} Root CA", organization),
            organization: organization.clone(),
            organizational_unit: "Root Certificate Authority".to_string(),
            country,
            state: "".to_string(),
            locality: "".to_string(),
            email: "root-ca@astor-currency.org".to_string(),
        };

        let extensions = CertificateExtensions {
            basic_constraints: Some(BasicConstraints {
                is_ca: true,
                path_length: Some(2), // Allow 2 levels below root
            }),
            key_usage: vec![
                KeyUsage::DigitalSignature,
                KeyUsage::KeyCertSign,
                KeyUsage::CrlSign,
            ],
            extended_key_usage: vec![],
            subject_alternative_names: vec![],
        };

        Ok(Self {
            version: 3,
            serial_number: "1".to_string(),
            issuer: subject.clone(), // Self-signed
            subject,
            public_key: public_key.as_bytes().to_vec(),
            not_before: now,
            not_after,
            certificate_type: CertificateType::RootCa,
            extensions,
            signature_algorithm: "Ed25519".to_string(),
            signature: vec![], // Will be filled by self-signing
            status: CertificateStatus::Valid,
        })
    }

    /// Create intermediate CA certificate
    pub fn new_intermediate_ca(
        public_key: PublicKey,
        ca_name: String,
        issuer_cert: Certificate,
        issuer_keypair: &KeyPair,
        serial_number: String,
        validity_years: u32,
    ) -> Result<Self, AstorError> {
        let now = Utc::now();
        let not_after = now + Duration::days(validity_years as i64 * 365);

        let subject = CertificateSubject {
            common_name: format!("{} Intermediate CA", ca_name),
            organization: issuer_cert.subject.organization.clone(),
            organizational_unit: "Intermediate Certificate Authority".to_string(),
            country: issuer_cert.subject.country.clone(),
            state: issuer_cert.subject.state.clone(),
            locality: issuer_cert.subject.locality.clone(),
            email: format!("intermediate-ca@astor-currency.org"),
        };

        let extensions = CertificateExtensions {
            basic_constraints: Some(BasicConstraints {
                is_ca: true,
                path_length: Some(0), // End-entity certificates only
            }),
            key_usage: vec![
                KeyUsage::DigitalSignature,
                KeyUsage::KeyCertSign,
                KeyUsage::CrlSign,
            ],
            extended_key_usage: vec![],
            subject_alternative_names: vec![],
        };

        let mut cert = Self {
            version: 3,
            serial_number,
            issuer: issuer_cert.subject,
            subject,
            public_key: public_key.as_bytes().to_vec(),
            not_before: now,
            not_after,
            certificate_type: CertificateType::IntermediateCa,
            extensions,
            signature_algorithm: "Ed25519".to_string(),
            signature: vec![],
            status: CertificateStatus::Valid,
        };

        // Sign certificate
        let signature = cert.sign_certificate(issuer_keypair)?;
        cert.signature = signature.to_base64().into_bytes();

        Ok(cert)
    }

    /// Create certificate from CSR
    pub fn from_csr(
        csr: CertificateSigningRequest,
        serial_number: String,
        issuer_cert: Certificate,
        issuer_keypair: &KeyPair,
        certificate_type: CertificateType,
        validity_days: u32,
    ) -> Result<Self, AstorError> {
        let now = Utc::now();
        let not_after = now + Duration::days(validity_days as i64);

        let extensions = match certificate_type {
            CertificateType::CurrencyNode => CertificateExtensions {
                basic_constraints: Some(BasicConstraints {
                    is_ca: false,
                    path_length: None,
                }),
                key_usage: vec![KeyUsage::DigitalSignature, KeyUsage::KeyEncipherment],
                extended_key_usage: vec![
                    ExtendedKeyUsage::ServerAuth,
                    ExtendedKeyUsage::ClientAuth,
                ],
                subject_alternative_names: csr.subject_alternative_names.clone(),
            },
            CertificateType::Bank => CertificateExtensions {
                basic_constraints: Some(BasicConstraints {
                    is_ca: false,
                    path_length: None,
                }),
                key_usage: vec![KeyUsage::DigitalSignature, KeyUsage::NonRepudiation],
                extended_key_usage: vec![ExtendedKeyUsage::ClientAuth],
                subject_alternative_names: csr.subject_alternative_names.clone(),
            },
            _ => {
                return Err(AstorError::InvalidOperation(
                    "Invalid certificate type for CSR".to_string(),
                ))
            }
        };

        let mut cert = Self {
            version: 3,
            serial_number,
            issuer: issuer_cert.subject,
            subject: csr.subject,
            public_key: csr.public_key,
            not_before: now,
            not_after,
            certificate_type,
            extensions,
            signature_algorithm: "Ed25519".to_string(),
            signature: vec![],
            status: CertificateStatus::Valid,
        };

        // Sign certificate
        let signature = cert.sign_certificate(issuer_keypair)?;
        cert.signature = signature.to_base64().into_bytes();

        Ok(cert)
    }

    /// Sign certificate with issuer's private key
    fn sign_certificate(&self, issuer_keypair: &KeyPair) -> Result<Signature, AstorError> {
        let tbs_certificate = self.to_be_signed_bytes()?;
        Ok(issuer_keypair.sign(&tbs_certificate))
    }

    /// Get certificate data to be signed
    fn to_be_signed_bytes(&self) -> Result<Vec<u8>, AstorError> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.version.to_be_bytes());
        data.extend_from_slice(self.serial_number.as_bytes());
        data.extend_from_slice(serde_json::to_string(&self.issuer)?.as_bytes());
        data.extend_from_slice(serde_json::to_string(&self.subject)?.as_bytes());
        data.extend_from_slice(&self.public_key);
        data.extend_from_slice(&self.not_before.timestamp().to_be_bytes());
        data.extend_from_slice(&self.not_after.timestamp().to_be_bytes());
        Ok(data)
    }

    /// Verify certificate signature
    pub fn verify_signature(&self, issuer_public_key: &PublicKey) -> Result<bool, AstorError> {
        let tbs_certificate = self.to_be_signed_bytes()?;
        let signature = Signature::from_base64(
            &String::from_utf8(self.signature.clone())?,
            "certificate_signature".to_string(),
        )?;

        match signature.verify(issuer_public_key, &tbs_certificate) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Check if certificate is currently valid
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        self.status == CertificateStatus::Valid && now >= self.not_before && now <= self.not_after
    }

    /// Get certificate public key
    pub fn public_key(&self) -> Result<PublicKey, AstorError> {
        PublicKey::from_bytes(&self.public_key)
            .map_err(|e| AstorError::CryptographicError(e.to_string()))
    }

    /// Export certificate as PEM format
    pub fn to_pem(&self) -> Result<String, AstorError> {
        let cert_data = serde_json::to_vec(self)?;
        let encoded = base64::encode(cert_data);

        Ok(format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
            encoded
        ))
    }

    // Getters
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }
    pub fn subject(&self) -> &CertificateSubject {
        &self.subject
    }
    pub fn issuer(&self) -> &CertificateSubject {
        &self.issuer
    }
    pub fn not_before(&self) -> DateTime<Utc> {
        self.not_before
    }
    pub fn not_after(&self) -> DateTime<Utc> {
        self.not_after
    }
    pub fn certificate_type(&self) -> &CertificateType {
        &self.certificate_type
    }
    pub fn status(&self) -> &CertificateStatus {
        &self.status
    }
}

/// Certificate types for different Astor Currency operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificateType {
    RootCa,
    IntermediateCa,
    CurrencyNode,
    Bank,
    Merchant,
    User,
    ApiClient,
}

/// Certificate status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificateStatus {
    Valid,
    Revoked,
    Expired,
    Suspended,
}

/// Certificate subject information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSubject {
    pub common_name: String,
    pub organization: String,
    pub organizational_unit: String,
    pub country: String,
    pub state: String,
    pub locality: String,
    pub email: String,
}

/// Certificate extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateExtensions {
    pub basic_constraints: Option<BasicConstraints>,
    pub key_usage: Vec<KeyUsage>,
    pub extended_key_usage: Vec<ExtendedKeyUsage>,
    pub subject_alternative_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicConstraints {
    pub is_ca: bool,
    pub path_length: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyUsage {
    DigitalSignature,
    NonRepudiation,
    KeyEncipherment,
    DataEncipherment,
    KeyAgreement,
    KeyCertSign,
    CrlSign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtendedKeyUsage {
    ServerAuth,
    ClientAuth,
    CodeSigning,
    EmailProtection,
    TimeStamping,
    OcspSigning,
}
