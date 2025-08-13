//! Input validation and security validation for Astor currency system

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::errors::AstorError;

/// Input validator for sanitizing and validating user inputs
pub struct InputValidator {
    email_regex: Regex,
    phone_regex: Regex,
    alphanumeric_regex: Regex,
    currency_code_regex: Regex,
    banned_patterns: HashSet<String>,
}

impl InputValidator {
    pub fn new() -> Result<Self, AstorError> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|e| AstorError::ValidationError(format!("Failed to compile email regex: {}", e)))?;
        
        let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$")
            .map_err(|e| AstorError::ValidationError(format!("Failed to compile phone regex: {}", e)))?;
        
        let alphanumeric_regex = Regex::new(r"^[a-zA-Z0-9_-]+$")
            .map_err(|e| AstorError::ValidationError(format!("Failed to compile alphanumeric regex: {}", e)))?;
        
        let currency_code_regex = Regex::new(r"^[A-Z]{3}$")
            .map_err(|e| AstorError::ValidationError(format!("Failed to compile currency code regex: {}", e)))?;

        let mut banned_patterns = HashSet::new();
        // Common XSS patterns
        banned_patterns.insert("<script".to_string());
        banned_patterns.insert("javascript:".to_string());
        banned_patterns.insert("onload=".to_string());
        banned_patterns.insert("onerror=".to_string());
        // SQL injection patterns
        banned_patterns.insert("' OR '1'='1".to_string());
        banned_patterns.insert("'; DROP TABLE".to_string());
        banned_patterns.insert("UNION SELECT".to_string());

        Ok(Self {
            email_regex,
            phone_regex,
            alphanumeric_regex,
            currency_code_regex,
            banned_patterns,
        })
    }

    /// Sanitize input by removing potentially dangerous characters
    pub fn sanitize_input(&self, input: &str) -> String {
        let mut sanitized = input
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('&', "&amp;");

        // Remove null bytes and control characters
        sanitized.retain(|c| c != '\0' && !c.is_control() || c == '\n' || c == '\r' || c == '\t');

        sanitized
    }

    /// Validate email format
    pub fn validate_email(&self, email: &str) -> Result<(), AstorError> {
        if email.is_empty() {
            return Err(AstorError::ValidationError("Email cannot be empty".to_string()));
        }

        if email.len() > 254 {
            return Err(AstorError::ValidationError("Email too long".to_string()));
        }

        if !self.email_regex.is_match(email) {
            return Err(AstorError::ValidationError("Invalid email format".to_string()));
        }

        self.check_for_malicious_patterns(email)?;
        Ok(())
    }

    /// Validate phone number format
    pub fn validate_phone(&self, phone: &str) -> Result<(), AstorError> {
        if phone.is_empty() {
            return Err(AstorError::ValidationError("Phone number cannot be empty".to_string()));
        }

        if !self.phone_regex.is_match(phone) {
            return Err(AstorError::ValidationError("Invalid phone number format".to_string()));
        }

        Ok(())
    }

    /// Validate currency amount
    pub fn validate_amount(&self, amount: i64) -> Result<(), AstorError> {
        if amount < 0 {
            return Err(AstorError::ValidationError("Amount cannot be negative".to_string()));
        }

        if amount > 1_000_000_000_000 { // 1 trillion limit
            return Err(AstorError::ValidationError("Amount exceeds maximum limit".to_string()));
        }

        Ok(())
    }

    /// Validate currency code (ISO 4217 format)
    pub fn validate_currency_code(&self, code: &str) -> Result<(), AstorError> {
        if !self.currency_code_regex.is_match(code) {
            return Err(AstorError::ValidationError("Invalid currency code format".to_string()));
        }

        Ok(())
    }

    /// Validate account ID format
    pub fn validate_account_id(&self, account_id: &str) -> Result<(), AstorError> {
        if account_id.is_empty() {
            return Err(AstorError::ValidationError("Account ID cannot be empty".to_string()));
        }

        if account_id.len() > 50 {
            return Err(AstorError::ValidationError("Account ID too long".to_string()));
        }

        if !self.alphanumeric_regex.is_match(account_id) {
            return Err(AstorError::ValidationError("Account ID contains invalid characters".to_string()));
        }

        Ok(())
    }

    /// Validate UUID format
    pub fn validate_uuid(&self, uuid_str: &str) -> Result<Uuid, AstorError> {
        Uuid::parse_str(uuid_str)
            .map_err(|_| AstorError::ValidationError("Invalid UUID format".to_string()))
    }

    /// Check for malicious patterns in input
    pub fn check_for_malicious_patterns(&self, input: &str) -> Result<(), AstorError> {
        let input_lower = input.to_lowercase();
        
        for pattern in &self.banned_patterns {
            if input_lower.contains(&pattern.to_lowercase()) {
                return Err(AstorError::SecurityViolation(
                    format!("Potentially malicious pattern detected: {}", pattern)
                ));
            }
        }

        Ok(())
    }

    /// Validate password strength
    pub fn validate_password(&self, password: &str) -> Result<(), AstorError> {
        if password.len() < 8 {
            return Err(AstorError::ValidationError("Password must be at least 8 characters".to_string()));
        }

        if password.len() > 128 {
            return Err(AstorError::ValidationError("Password too long".to_string()));
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        if !has_uppercase || !has_lowercase || !has_digit || !has_special {
            return Err(AstorError::ValidationError(
                "Password must contain uppercase, lowercase, digit, and special character".to_string()
            ));
        }

        Ok(())
    }
}

/// Security validator for business logic and security rules
pub struct SecurityValidator {
    max_transaction_amount: i64,
    max_daily_transaction_amount: i64,
    allowed_currencies: HashSet<String>,
}

impl SecurityValidator {
    pub fn new() -> Self {
        let mut allowed_currencies = HashSet::new();
        allowed_currencies.insert("USD".to_string());
        allowed_currencies.insert("EUR".to_string());
        allowed_currencies.insert("GBP".to_string());
        allowed_currencies.insert("JPY".to_string());
        allowed_currencies.insert("CAD".to_string());
        allowed_currencies.insert("AUD".to_string());
        allowed_currencies.insert("CHF".to_string());
        allowed_currencies.insert("CNY".to_string());
        allowed_currencies.insert("AST".to_string()); // Astor currency

        Self {
            max_transaction_amount: 1_000_000_00, // $1M in cents
            max_daily_transaction_amount: 10_000_000_00, // $10M in cents
            allowed_currencies,
        }
    }

    /// Validate transaction amount limits
    pub fn validate_transaction_limits(&self, amount: i64) -> Result<(), AstorError> {
        if amount > self.max_transaction_amount {
            return Err(AstorError::ValidationError(
                format!("Transaction amount {} exceeds maximum limit {}", 
                    amount, self.max_transaction_amount)
            ));
        }

        Ok(())
    }

    /// Validate daily transaction limits
    pub fn validate_daily_limits(&self, daily_total: i64, new_amount: i64) -> Result<(), AstorError> {
        if daily_total + new_amount > self.max_daily_transaction_amount {
            return Err(AstorError::ValidationError(
                "Daily transaction limit exceeded".to_string()
            ));
        }

        Ok(())
    }

    /// Validate currency is supported
    pub fn validate_currency_support(&self, currency: &str) -> Result<(), AstorError> {
        if !self.allowed_currencies.contains(currency) {
            return Err(AstorError::ValidationError(
                format!("Currency {} is not supported", currency)
            ));
        }

        Ok(())
    }

    /// Validate account balance for transaction
    pub fn validate_sufficient_balance(&self, balance: i64, amount: i64) -> Result<(), AstorError> {
        if balance < amount {
            return Err(AstorError::ValidationError(
                "Insufficient balance for transaction".to_string()
            ));
        }

        Ok(())
    }

    /// Validate transaction frequency (anti-spam)
    pub fn validate_transaction_frequency(&self, recent_transactions: u32, max_per_minute: u32) -> Result<(), AstorError> {
        if recent_transactions >= max_per_minute {
            return Err(AstorError::ValidationError(
                "Transaction frequency limit exceeded".to_string()
            ));
        }

        Ok(())
    }
}

/// Validation result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// Comprehensive validation for transaction data
pub fn validate_transaction_data(
    from_account: &str,
    to_account: &str,
    amount: i64,
    currency: &str,
    validator: &InputValidator,
    security_validator: &SecurityValidator,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Validate account IDs
    if let Err(e) = validator.validate_account_id(from_account) {
        result.add_error(format!("From account: {}", e));
    }

    if let Err(e) = validator.validate_account_id(to_account) {
        result.add_error(format!("To account: {}", e));
    }

    // Validate amount
    if let Err(e) = validator.validate_amount(amount) {
        result.add_error(format!("Amount: {}", e));
    }

    if let Err(e) = security_validator.validate_transaction_limits(amount) {
        result.add_error(format!("Transaction limits: {}", e));
    }

    // Validate currency
    if let Err(e) = validator.validate_currency_code(currency) {
        result.add_error(format!("Currency code: {}", e));
    }

    if let Err(e) = security_validator.validate_currency_support(currency) {
        result.add_error(format!("Currency support: {}", e));
    }

    // Check for same account transfer
    if from_account == to_account {
        result.add_error("Cannot transfer to the same account".to_string());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = InputValidator::new().unwrap();
        
        assert!(validator.validate_email("test@example.com").is_ok());
        assert!(validator.validate_email("invalid-email").is_err());
        assert!(validator.validate_email("").is_err());
    }

    #[test]
    fn test_amount_validation() {
        let validator = InputValidator::new().unwrap();
        
        assert!(validator.validate_amount(100).is_ok());
        assert!(validator.validate_amount(-100).is_err());
        assert!(validator.validate_amount(1_000_000_000_001).is_err());
    }

    #[test]
    fn test_malicious_pattern_detection() {
        let validator = InputValidator::new().unwrap();
        
        assert!(validator.check_for_malicious_patterns("normal text").is_ok());
        assert!(validator.check_for_malicious_patterns("<script>alert('xss')</script>").is_err());
        assert!(validator.check_for_malicious_patterns("'; DROP TABLE users; --").is_err());
    }

    #[test]
    fn test_password_validation() {
        let validator = InputValidator::new().unwrap();
        
        assert!(validator.validate_password("StrongP@ss1").is_ok());
        assert!(validator.validate_password("weak").is_err());
        assert!(validator.validate_password("NoSpecialChar1").is_err());
    }
}
