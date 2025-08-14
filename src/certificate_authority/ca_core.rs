//! Core Certificate Authority implementation

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::errors::AstorError;
use crate::security::KeyPair;
use super::certificate::{Certificate, CertificateType};
use super::csr::CertificateSigningRequest;

/// Certificate Authority core implementation
#[derive(Clone)]
pub struct CertificateAuthority {
    ca_id: uuid::Uuid,
    ca_certificate: Certificate,
    ca_keypair: KeyPair,
    config: CaConfig,
    issued_certificates: HashMap<String, Certificate>,
    serial_counter: u64,
}

impl CertificateAuthority {
    /// Create new root Certificate Authority
    pub fn new_root(keypair: KeyPair, config: CaConfig) -> Result<Self, AstorError> {
        let ca_id = uuid::Uuid::new_v4();
        
        // Create self-signed root certificate
        let ca_certificate = Certificate::new_root_ca(
            keypair.public_key(),
            config.organization.clone(),
            config.country.clone(),
            config.validity_years,
        )?;

        Ok(Self {
            ca_id,
            ca_certificate,
            ca_keypair: keypair,
            config,
            issued_certificates: HashMap::new(),
            serial_counter: 1,
        })
    }

    /// Create intermediate Certificate Authority
    pub async fn create_intermediate_ca(
        &self,
        ca_name: String,
        keypair: KeyPair,
        config: CaConfig,
    ) -> Result<CertificateAuthority, AstorError> {
        let ca_id = uuid::Uuid::new_v4();
        
        // Create intermediate CA certificate signed by this CA
        let ca_certificate = self.sign_intermediate_ca_certificate(
            keypair.public_key(),
            ca_name,
            config.validity_years,
        ).await?;

        Ok(CertificateAuthority {
            ca_id,
            ca_certificate,
            ca_keypair: keypair,
            config,
            issued_certificates: HashMap::new(),
            serial_counter: 1,
        })
    }

    /// Issue a certificate from CSR
    pub async fn issue_certificate(
        &self,
        csr: CertificateSigningRequest,
        certificate_type: CertificateType,
        validity_days: u32,
    ) -> Result<Certificate, AstorError> {
        let serial_number = self.generate_serial_number();
        
        let certificate = Certificate::from_csr(
            csr,
            serial_number,
            self.ca_certificate.clone(),
            &self.ca_keypair,
            certificate_type,
            validity_days,
        )?;

        tracing::info!(
            "Certificate issued by CA {}: serial={}",
            self.ca_id,
            certificate.serial_number()
        );

        Ok(certificate)
    }

    /// Sign intermediate CA certificate
    async fn sign_intermediate_ca_certificate(
        &self,
        public_key: ed25519_dalek::PublicKey,
        ca_name: String,
        validity_years: u32,
    ) -> Result<Certificate, AstorError> {
        let serial_number = self.generate_serial_number();
        
        Certificate::new_intermediate_ca(
            public_key,
            ca_name,
            self.ca_certificate.clone(),
            &self.ca_keypair,
            serial_number,
            validity_years,
        )
    }

    /// Generate unique serial number
    fn generate_serial_number(&self) -> String {
        format!("{:016X}", self.serial_counter)
    }

    /// Get CA certificate
    pub fn get_certificate(&self) -> &Certificate {
        &self.ca_certificate
    }

    /// Get CA ID
    pub fn get_ca_id(&self) -> uuid::Uuid {
        self.ca_id
    }

    /// Verify certificate was issued by this CA
    pub fn verify_issued_certificate(&self, certificate: &Certificate) -> Result<bool, AstorError> {
        certificate.verify_signature(&self.ca_certificate.public_key())
    }
}

/// Certificate Authority configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaConfig {
    pub organization: String,
    pub organizational_unit: String,
    pub country: String,
    pub state: String,
    pub locality: String,
    pub email: String,
    pub validity_years: u32,
    pub key_usage: Vec<String>,
    pub extended_key_usage: Vec<String>,
}

impl Default for CaConfig {
    fn default() -> Self {
        Self {
            organization: "Astor Digital Currency Authority".to_string(),
            organizational_unit: "Certificate Authority".to_string(),
            country: "US".to_string(),
            state: "California".to_string(),
            locality: "San Francisco".to_string(),
            email: "ca@astor-currency.org".to_string(),
            validity_years: 10,
            key_usage: vec![
                "digitalSignature".to_string(),
                "keyCertSign".to_string(),
                "cRLSign".to_string(),
            ],
            extended_key_usage: vec![
                "serverAuth".to_string(),
                "clientAuth".to_string(),
            ],
        }
    }
}
