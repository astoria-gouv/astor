//! Enhanced security module for production-grade protection

pub mod crypto;
pub mod auth;
pub mod validation;
pub mod audit;
pub mod session;
pub mod encryption;
pub mod fraud_detection;

pub use crypto::{KeyPair, Signature, hash_data};
pub use auth::{Role, Permission, AccessControl};
pub use validation::{InputValidator, SecurityValidator};
pub use audit::{SecurityAuditLogger, SecurityEvent};
pub use session::{SessionManager, Session};
pub use encryption::{EncryptionManager, EncryptedData};
pub use fraud_detection::{FraudDetector, RiskScore};

use crate::errors::AstorError;

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub refresh_token_expiration: i64,
    pub bcrypt_cost: u32,
    pub max_login_attempts: u32,
    pub lockout_duration: i64,
    pub session_timeout: i64,
    pub require_mfa: bool,
    pub encryption_key: String,
}

/// Main security manager
pub struct SecurityManager {
    config: SecurityConfig,
    session_manager: SessionManager,
    audit_logger: SecurityAuditLogger,
    fraud_detector: FraudDetector,
    encryption_manager: EncryptionManager,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Result<Self, AstorError> {
        let session_manager = SessionManager::new(config.session_timeout);
        let audit_logger = SecurityAuditLogger::new();
        let fraud_detector = FraudDetector::new();
        let encryption_manager = EncryptionManager::new(&config.encryption_key)?;

        Ok(Self {
            config,
            session_manager,
            audit_logger,
            fraud_detector,
            encryption_manager,
        })
    }

    /// Comprehensive security check for operations
    pub async fn security_check(
        &mut self,
        user_id: &str,
        operation: &str,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<(), AstorError> {
        // Check for fraud patterns
        let risk_score = self.fraud_detector.assess_risk(user_id, operation, ip_address).await?;
        if risk_score.is_high_risk() {
            self.audit_logger.log_security_event(SecurityEvent::HighRiskOperation {
                user_id: user_id.to_string(),
                operation: operation.to_string(),
                risk_score: risk_score.score(),
                ip_address: ip_address.to_string(),
            }).await?;
            return Err(AstorError::SecurityViolation("High risk operation detected".to_string()));
        }

        // Validate session
        if !self.session_manager.is_valid_session(user_id).await? {
            return Err(AstorError::Unauthorized("Invalid session".to_string()));
        }

        Ok(())
    }
}
