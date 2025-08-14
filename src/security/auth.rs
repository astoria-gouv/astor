//! Enhanced authentication and authorization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::errors::AstorError;

/// Enhanced role-based access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Role {
    RootAdmin,
    CentralBankAdmin,
    BankAdmin,
    Auditor,
    Operator,
    User,
}

impl Role {
    /// Get all permissions for this role
    pub fn permissions(&self) -> HashSet<Permission> {
        match self {
            Role::RootAdmin => {
                let mut perms = HashSet::new();
                perms.insert(Permission::IssueCurrency);
                perms.insert(Permission::ManageAdmins);
                perms.insert(Permission::ViewAuditLogs);
                perms.insert(Permission::ManageAccounts);
                perms.insert(Permission::FreezeAccounts);
                perms.insert(Permission::SystemConfiguration);
                perms.insert(Permission::EmergencyShutdown);
                perms
            }
            Role::CentralBankAdmin => {
                let mut perms = HashSet::new();
                perms.insert(Permission::IssueCurrency);
                perms.insert(Permission::ViewAuditLogs);
                perms.insert(Permission::ManageAccounts);
                perms.insert(Permission::FreezeAccounts);
                perms
            }
            Role::BankAdmin => {
                let mut perms = HashSet::new();
                perms.insert(Permission::ManageAccounts);
                perms.insert(Permission::ViewTransactions);
                perms
            }
            Role::Auditor => {
                let mut perms = HashSet::new();
                perms.insert(Permission::ViewAuditLogs);
                perms.insert(Permission::ViewTransactions);
                perms.insert(Permission::ViewAccounts);
                perms
            }
            Role::Operator => {
                let mut perms = HashSet::new();
                perms.insert(Permission::ViewTransactions);
                perms.insert(Permission::ViewAccounts);
                perms
            }
            Role::User => HashSet::new(),
        }
    }

    /// Check if role has specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions().contains(permission)
    }
}

/// Granular permissions system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    IssueCurrency,
    ManageAdmins,
    ViewAuditLogs,
    ManageAccounts,
    FreezeAccounts,
    ViewTransactions,
    ViewAccounts,
    SystemConfiguration,
    EmergencyShutdown,
}

/// Access control manager
pub struct AccessControl {
    user_roles: std::collections::HashMap<Uuid, HashSet<Role>>,
    user_permissions: std::collections::HashMap<Uuid, HashSet<Permission>>,
}

impl AccessControl {
    pub fn new() -> Self {
        Self {
            user_roles: std::collections::HashMap::new(),
            user_permissions: std::collections::HashMap::new(),
        }
    }

    /// Assign role to user
    pub fn assign_role(&mut self, user_id: Uuid, role: Role) {
        self.user_roles
            .entry(user_id)
            .or_insert_with(HashSet::new)
            .insert(role);
    }

    /// Grant specific permission to user
    pub fn grant_permission(&mut self, user_id: Uuid, permission: Permission) {
        self.user_permissions
            .entry(user_id)
            .or_insert_with(HashSet::new)
            .insert(permission);
    }

    /// Check if user has permission
    pub fn has_permission(&self, user_id: Uuid, permission: &Permission) -> bool {
        // Check direct permissions
        if let Some(perms) = self.user_permissions.get(&user_id) {
            if perms.contains(permission) {
                return true;
            }
        }

        // Check role-based permissions
        if let Some(roles) = self.user_roles.get(&user_id) {
            for role in roles {
                if role.has_permission(permission) {
                    return true;
                }
            }
        }

        false
    }

    /// Get all permissions for user
    pub fn get_user_permissions(&self, user_id: Uuid) -> HashSet<Permission> {
        let mut all_permissions = HashSet::new();

        // Add direct permissions
        if let Some(perms) = self.user_permissions.get(&user_id) {
            all_permissions.extend(perms.clone());
        }

        // Add role-based permissions
        if let Some(roles) = self.user_roles.get(&user_id) {
            for role in roles {
                all_permissions.extend(role.permissions());
            }
        }

        all_permissions
    }
}

/// Multi-factor authentication manager
pub struct MfaManager {
    user_secrets: std::collections::HashMap<Uuid, Vec<u8>>,
    backup_codes: std::collections::HashMap<Uuid, HashSet<String>>,
}

impl MfaManager {
    pub fn new() -> Self {
        Self {
            user_secrets: std::collections::HashMap::new(),
            backup_codes: std::collections::HashMap::new(),
        }
    }

    /// Enable MFA for user
    pub fn enable_mfa(&mut self, user_id: Uuid) -> Result<String, AstorError> {
        let secret = crate::security::crypto::generate_secure_random(32);
        let secret_base32 = base64::encode(&secret);

        self.user_secrets.insert(user_id, secret);

        // Generate backup codes
        let mut backup_codes = HashSet::new();
        for _ in 0..10 {
            let code = crate::security::crypto::generate_api_key()[..8].to_string();
            backup_codes.insert(code);
        }
        self.backup_codes.insert(user_id, backup_codes.clone());

        Ok(secret_base32)
    }

    /// Verify MFA code
    pub fn verify_mfa(&self, user_id: Uuid, code: &str) -> bool {
        if let Some(secret) = self.user_secrets.get(&user_id) {
            let totp = crate::security::crypto::TotpGenerator {
                secret: secret.clone(),
            };
            if totp.verify_code(code, 1) {
                return true;
            }
        }

        // Check backup codes
        if let Some(backup_codes) = self.backup_codes.get(&user_id) {
            return backup_codes.contains(code);
        }

        false
    }
}

/// Login attempt tracking for brute force protection
#[derive(Debug, Clone)]
pub struct LoginAttempt {
    pub user_id: String,
    pub ip_address: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub user_agent: Option<String>,
}

pub struct BruteForceProtection {
    attempts: Vec<LoginAttempt>,
    max_attempts: u32,
    lockout_duration: chrono::Duration,
}

impl BruteForceProtection {
    pub fn new(max_attempts: u32, lockout_duration_minutes: i64) -> Self {
        Self {
            attempts: Vec::new(),
            max_attempts,
            lockout_duration: chrono::Duration::minutes(lockout_duration_minutes),
        }
    }

    /// Record login attempt
    pub fn record_attempt(&mut self, attempt: LoginAttempt) {
        // Clean old attempts
        let cutoff = Utc::now() - self.lockout_duration;
        self.attempts.retain(|a| a.timestamp > cutoff);

        self.attempts.push(attempt);
    }

    /// Check if user/IP is locked out
    pub fn is_locked_out(&self, user_id: &str, ip_address: &str) -> bool {
        let cutoff = Utc::now() - self.lockout_duration;

        let failed_attempts = self
            .attempts
            .iter()
            .filter(|a| {
                a.timestamp > cutoff
                    && !a.success
                    && (a.user_id == user_id || a.ip_address == ip_address)
            })
            .count();

        failed_attempts >= self.max_attempts as usize
    }
}
