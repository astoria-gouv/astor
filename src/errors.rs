//! Error types for the Astor currency system

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AstorError {
    #[error("Unauthorized access: {0}")]
    Unauthorized(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Administrator not found: {0}")]
    AdminNotFound(String),

    #[error("Insufficient funds for transaction")]
    InsufficientFunds,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Transaction validation failed: {0}")]
    TransactionValidationFailed(String),

    #[error("Ledger error: {0}")]
    LedgerError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Cryptographic error: {0}")]
    CryptographicError(String),

    #[error("Central bank error: {0}")]
    CentralBankError(String),

    #[error("Commercial banking error: {0}")]
    CommercialBankingError(String),

    #[error("Payment processing error: {0}")]
    PaymentError(String),

    #[error("Regulatory compliance error: {0}")]
    ComplianceError(String),

    #[error("KYC verification failed: {0}")]
    KycError(String),

    #[error("AML violation detected: {0}")]
    AmlViolation(String),

    #[error("Tax reporting error: {0}")]
    TaxReportingError(String),

    #[error("Loan processing error: {0}")]
    LoanError(String),

    #[error("Credit line error: {0}")]
    CreditError(String),

    #[error("Interest calculation error: {0}")]
    InterestCalculationError(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}
