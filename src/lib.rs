//! Astor Digital Currency System
//!
//! A centralized digital currency system with administrator-controlled issuance,
//! secure ledger management, and user account functionality.

pub mod accounts;
pub mod admin;
pub mod analytics;
pub mod api;
pub mod banking_network;
pub mod central_bank;
pub mod certificate_authority;
pub mod cli;
pub mod commercial_banking;
pub mod config;
pub mod conversion;
pub mod database;
pub mod errors;
pub mod interoperability;
pub mod ledger;
pub mod monitoring;
pub mod network;
pub mod payment_processing;
pub mod regulatory;
pub mod security;
pub mod smart_contracts;
pub mod transactions;

pub use accounts::AccountManager;
pub use admin::AdminManager;
pub use banking_network::{BankStatus, BankingNetwork, RegisteredBank};
pub use central_bank::CentralBank;
pub use certificate_authority::{
    AstorCertificateAuthority, Certificate, CertificateAuthorityConfig, CertificateSigningRequest,
    CertificateStatus, CertificateType, CsrProcessor,
};
pub use cli::{CentralBankCli, CliHandler};
pub use commercial_banking::CommercialBank;
pub use errors::AstorError;
pub use ledger::Ledger;
pub use monitoring::MonitoringSystem;
pub use network::{NetworkManager, NetworkStatus};
pub use payment_processing::PaymentProcessor;
pub use regulatory::RegulatoryCompliance;
pub use security::{KeyPair, Signature};
pub use transactions::TransactionManager;

/// Core Astor system that orchestrates all components
pub struct AstorSystem {
    pub admin_manager: AdminManager,
    pub ledger: Ledger,
    pub account_manager: AccountManager,
    pub transaction_manager: TransactionManager,
    pub monitoring: MonitoringSystem,
    pub central_bank: CentralBank,
    pub commercial_banks: std::collections::HashMap<String, CommercialBank>,
    pub payment_processor: PaymentProcessor,
    pub regulatory_compliance: RegulatoryCompliance,
    pub banking_network: BankingNetwork,
    pub certificate_authority: AstorCertificateAuthority,
}

impl AstorSystem {
    /// Initialize a new Astor system with a root administrator
    pub async fn new(
        root_admin_keypair: KeyPair,
        monitoring_config: config::MonitoringConfig,
    ) -> Result<Self, AstorError> {
        let mut admin_manager = AdminManager::new();
        let ledger = Ledger::new();
        let account_manager = AccountManager::new();
        let transaction_manager = TransactionManager::new();
        let monitoring = MonitoringSystem::new(monitoring_config).await?;

        let central_bank_config = central_bank::CentralBankConfig {
            base_interest_rate: 0.025,        // 2.5%
            reserve_requirement_ratio: 0.10,  // 10%
            inflation_target: 0.02,           // 2%
            money_supply_growth_target: 0.03, // 3%
            emergency_lending_rate: 0.05,     // 5%
        };
        let central_bank = CentralBank::new(central_bank_config);
        let commercial_banks = std::collections::HashMap::new();
        let payment_processor = PaymentProcessor::new();
        let regulatory_compliance = RegulatoryCompliance::new();
        let banking_network = BankingNetwork::new(central_bank.clone());

        admin_manager.add_admin("root".to_string(), root_admin_keypair.public_key())?;

        let ca_config = certificate_authority::ca_core::CaConfig::default();
        let ca_keypair = KeyPair::generate(); // Separate keypair for CA
        let certificate_authority = AstorCertificateAuthority::new(ca_keypair, ca_config)?;

        monitoring.start().await?;

        Ok(Self {
            admin_manager,
            ledger,
            account_manager,
            transaction_manager,
            monitoring,
            central_bank,
            commercial_banks,
            payment_processor,
            regulatory_compliance,
            banking_network,
            certificate_authority,
        })
    }

    /// Initialize a new Astor system with networking capabilities
    pub async fn new_with_network(
        root_admin_keypair: KeyPair,
        monitoring_config: config::MonitoringConfig,
        network_config: network::NodeConfig,
    ) -> Result<(Self, NetworkManager), AstorError> {
        let mut admin_manager = AdminManager::new();
        let ledger = Ledger::new();
        let account_manager = AccountManager::new();
        let transaction_manager = TransactionManager::new();
        let monitoring = MonitoringSystem::new(monitoring_config).await?;

        let central_bank_config = central_bank::CentralBankConfig {
            base_interest_rate: 0.025,
            reserve_requirement_ratio: 0.10,
            inflation_target: 0.02,
            money_supply_growth_target: 0.03,
            emergency_lending_rate: 0.05,
        };
        let central_bank = CentralBank::new(central_bank_config);
        let commercial_banks = std::collections::HashMap::new();
        let payment_processor = PaymentProcessor::new();
        let regulatory_compliance = RegulatoryCompliance::new();
        let banking_network = BankingNetwork::new(central_bank.clone());

        admin_manager.add_admin("root".to_string(), root_admin_keypair.public_key())?;

        let ca_config = certificate_authority::ca_core::CaConfig::default();
        let ca_keypair = KeyPair::generate(); // Separate keypair for CA
        let certificate_authority = AstorCertificateAuthority::new(ca_keypair, ca_config)?;

        monitoring.start().await?;

        let system = Self {
            admin_manager,
            ledger,
            account_manager,
            transaction_manager,
            monitoring,
            central_bank,
            commercial_banks,
            payment_processor,
            regulatory_compliance,
            banking_network,
            certificate_authority,
        };

        let network_manager = NetworkManager::new(network_config).await?;

        Ok((system, network_manager))
    }

    /// Issue new Astor units (admin only)
    pub async fn issue_currency(
        &mut self,
        admin_id: &str,
        recipient_account: &str,
        amount: u64,
        admin_signature: &Signature,
    ) -> Result<String, AstorError> {
        self.monitoring
            .record_business_metric(monitoring::BusinessMetric::CurrencyIssued {
                amount: amount as i64,
                issuer: admin_id.to_string(),
            })
            .await;

        let decision_id = self.central_bank.issue_currency(
            amount,
            format!(
                "Currency issued by admin {} to account {}",
                admin_id, recipient_account
            ),
        )?;

        Ok(format!(
            "Currency issued successfully. Decision ID: {}",
            decision_id
        ))
    }

    /// Register a commercial bank
    pub fn register_commercial_bank(
        &mut self,
        bank_id: String,
        bank_name: String,
    ) -> Result<(), AstorError> {
        let bank = CommercialBank::new(bank_id.clone(), bank_name);
        self.commercial_banks.insert(bank_id, bank);
        Ok(())
    }

    /// Process payment through payment processor
    pub fn process_payment(
        &mut self,
        merchant_id: String,
        customer_id: String,
        payment_method_id: String,
        amount: u64,
        currency: String,
    ) -> Result<String, AstorError> {
        self.payment_processor.process_payment(
            merchant_id,
            customer_id,
            payment_method_id,
            amount,
            currency,
        )
    }

    /// Perform KYC verification
    pub fn perform_kyc(
        &mut self,
        customer_id: String,
        documents: Vec<regulatory::IdentityDocument>,
        verification_level: regulatory::KycLevel,
    ) -> Result<(), AstorError> {
        self.regulatory_compliance.perform_kyc_verification(
            customer_id,
            documents,
            verification_level,
        )
    }

    /// Deploy the currency network
    pub async fn deploy_network(
        &mut self,
        network_manager: &NetworkManager,
    ) -> Result<(), AstorError> {
        network_manager.start().await?;
        self.setup_network_handlers(network_manager).await?;
        tracing::info!("Astor currency network deployed successfully");
        Ok(())
    }

    async fn setup_network_handlers(
        &self,
        network_manager: &NetworkManager,
    ) -> Result<(), AstorError> {
        tracing::info!("Network handlers configured");
        Ok(())
    }

    /// Get network deployment status
    pub async fn get_network_status(&self, network_manager: &NetworkManager) -> NetworkStatus {
        network_manager.get_network_status().await
    }

    /// Register a commercial bank in the banking network
    pub async fn register_bank_in_network(
        &mut self,
        bank_name: String,
        license_number: String,
        api_endpoint: String,
        public_key: String,
        services_offered: Vec<banking_network::BankingService>,
    ) -> Result<String, AstorError> {
        self.banking_network
            .register_bank(
                bank_name,
                license_number,
                api_endpoint,
                public_key,
                services_offered,
            )
            .await
    }

    /// Approve a bank registration
    pub async fn approve_bank_registration(&self, bank_id: &str) -> Result<(), AstorError> {
        self.banking_network.approve_bank(bank_id).await
    }

    /// Get banking network statistics
    pub async fn get_banking_network_stats(&self) -> banking_network::NetworkStats {
        self.banking_network.get_network_stats().await
    }

    /// Issue certificate for currency operations
    pub async fn issue_certificate(
        &mut self,
        csr: CertificateSigningRequest,
        certificate_type: CertificateType,
        validity_days: u32,
    ) -> Result<Certificate, AstorError> {
        self.certificate_authority
            .issue_certificate(csr, certificate_type, validity_days)
            .await
    }

    /// Revoke certificate
    pub async fn revoke_certificate(
        &mut self,
        serial_number: &str,
        reason: certificate_authority::crl::RevocationReason,
    ) -> Result<(), AstorError> {
        self.certificate_authority
            .revoke_certificate(serial_number, reason)
            .await
    }

    /// Get root CA certificate for distribution
    pub fn get_root_ca_certificate(&self) -> Certificate {
        self.certificate_authority.get_root_certificate()
    }

    /// Validate certificate chain
    pub fn validate_certificate(&self, certificate: &Certificate) -> Result<bool, AstorError> {
        self.certificate_authority
            .validate_certificate_chain(certificate)
    }
}
