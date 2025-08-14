//! Certificate Authority module for Astor Currency System
//! 
//! Provides PKI functionality similar to OpenSSL for HTTPS certificates,
//! enabling centralized certificate issuance and management for currency operations.

pub mod ca_core;
pub mod certificate;
pub mod csr;
pub mod crl;
pub mod ocsp;
pub mod pki_hierarchy;

pub use ca_core::{CertificateAuthority, CaConfig};
pub use certificate::{Certificate, CertificateType, CertificateStatus};
pub use csr::{CertificateSigningRequest, CsrProcessor};
pub use crl::{CertificateRevocationList, RevocationReason};
pub use ocsp::{OcspResponder, OcspRequest, OcspResponse};
pub use pki_hierarchy::{PkiHierarchy, CaLevel};

use crate::errors::AstorError;
use crate::security::KeyPair;

/// Main Certificate Authority System for Astor Currency
pub struct AstorCertificateAuthority {
    root_ca: CertificateAuthority,
    intermediate_cas: std::collections::HashMap<String, CertificateAuthority>,
    pki_hierarchy: PkiHierarchy,
    csr_processor: CsrProcessor,
    crl_manager: CertificateRevocationList,
    ocsp_responder: OcspResponder,
}

impl AstorCertificateAuthority {
    /// Initialize new Certificate Authority system
    pub fn new(root_keypair: KeyPair, ca_config: CaConfig) -> Result<Self, AstorError> {
        let root_ca = CertificateAuthority::new_root(root_keypair, ca_config.clone())?;
        let intermediate_cas = std::collections::HashMap::new();
        let pki_hierarchy = PkiHierarchy::new(root_ca.get_certificate().clone());
        let csr_processor = CsrProcessor::new();
        let crl_manager = CertificateRevocationList::new(root_ca.get_certificate().clone());
        let ocsp_responder = OcspResponder::new(root_ca.get_certificate().clone());

        Ok(Self {
            root_ca,
            intermediate_cas,
            pki_hierarchy,
            csr_processor,
            crl_manager,
            ocsp_responder,
        })
    }

    /// Issue a new certificate for currency operations
    pub async fn issue_certificate(
        &mut self,
        csr: CertificateSigningRequest,
        certificate_type: CertificateType,
        validity_days: u32,
    ) -> Result<Certificate, AstorError> {
        // Validate CSR
        self.csr_processor.validate_csr(&csr)?;

        // Determine issuing CA based on certificate type
        let issuing_ca = match certificate_type {
            CertificateType::RootCa => return Err(AstorError::InvalidOperation("Cannot issue root CA certificate".to_string())),
            CertificateType::IntermediateCa => &self.root_ca,
            _ => self.get_appropriate_intermediate_ca(&certificate_type)?,
        };

        // Issue certificate
        let certificate = issuing_ca.issue_certificate(csr, certificate_type, validity_days).await?;

        // Add to PKI hierarchy
        self.pki_hierarchy.add_certificate(certificate.clone())?;

        // Log certificate issuance
        tracing::info!(
            "Certificate issued: serial={}, type={:?}, subject={}",
            certificate.serial_number(),
            certificate_type,
            certificate.subject()
        );

        Ok(certificate)
    }

    /// Create intermediate Certificate Authority
    pub async fn create_intermediate_ca(
        &mut self,
        ca_name: String,
        keypair: KeyPair,
        config: CaConfig,
    ) -> Result<String, AstorError> {
        let intermediate_ca = self.root_ca.create_intermediate_ca(
            ca_name.clone(),
            keypair,
            config,
        ).await?;

        let ca_id = intermediate_ca.get_ca_id().to_string();
        self.intermediate_cas.insert(ca_id.clone(), intermediate_ca);

        tracing::info!("Intermediate CA created: {}", ca_name);
        Ok(ca_id)
    }

    /// Revoke a certificate
    pub async fn revoke_certificate(
        &mut self,
        serial_number: &str,
        reason: RevocationReason,
    ) -> Result<(), AstorError> {
        // Add to CRL
        self.crl_manager.revoke_certificate(serial_number, reason).await?;

        // Update OCSP responder
        self.ocsp_responder.mark_revoked(serial_number, reason).await?;

        tracing::warn!("Certificate revoked: serial={}, reason={:?}", serial_number, reason);
        Ok(())
    }

    /// Validate certificate chain
    pub fn validate_certificate_chain(&self, certificate: &Certificate) -> Result<bool, AstorError> {
        self.pki_hierarchy.validate_chain(certificate)
    }

    /// Get Certificate Revocation List
    pub async fn get_crl(&self) -> Result<Vec<u8>, AstorError> {
        self.crl_manager.generate_crl().await
    }

    /// Handle OCSP request
    pub async fn handle_ocsp_request(&self, request: OcspRequest) -> Result<OcspResponse, AstorError> {
        self.ocsp_responder.handle_request(request).await
    }

    /// Get CA certificate for distribution
    pub fn get_root_certificate(&self) -> Certificate {
        self.root_ca.get_certificate().clone()
    }

    /// Get intermediate CA certificate
    pub fn get_intermediate_certificate(&self, ca_id: &str) -> Result<Certificate, AstorError> {
        self.intermediate_cas
            .get(ca_id)
            .map(|ca| ca.get_certificate().clone())
            .ok_or_else(|| AstorError::NotFound(format!("Intermediate CA not found: {}", ca_id)))
    }

    /// List all certificates
    pub fn list_certificates(&self) -> Vec<Certificate> {
        self.pki_hierarchy.list_all_certificates()
    }

    /// Get certificate by serial number
    pub fn get_certificate(&self, serial_number: &str) -> Result<Certificate, AstorError> {
        self.pki_hierarchy.get_certificate(serial_number)
    }

    fn get_appropriate_intermediate_ca(&self, cert_type: &CertificateType) -> Result<&CertificateAuthority, AstorError> {
        // For now, use root CA for all non-intermediate certificates
        // In production, you might have specialized intermediate CAs for different purposes
        Ok(&self.root_ca)
    }
}

/// Certificate Authority configuration
#[derive(Debug, Clone)]
pub struct CertificateAuthorityConfig {
    pub ca_config: CaConfig,
    pub default_validity_days: u32,
    pub max_validity_days: u32,
    pub crl_update_interval_hours: u32,
    pub ocsp_responder_url: String,
    pub enable_key_escrow: bool,
}

impl Default for CertificateAuthorityConfig {
    fn default() -> Self {
        Self {
            ca_config: CaConfig::default(),
            default_validity_days: 365,
            max_validity_days: 3650, // 10 years
            crl_update_interval_hours: 24,
            ocsp_responder_url: "http://ocsp.astor-currency.org".to_string(),
            enable_key_escrow: false,
        }
    }
}
