//! Administrator management module

use std::collections::HashMap;
use ed25519_dalek::PublicKey;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::security::{Role, Signature};
use crate::errors::AstorError;

/// Administrator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Administrator {
    pub id: String,
    pub public_key: PublicKey,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Manages system administrators
pub struct AdminManager {
    admins: HashMap<String, Administrator>,
}

impl AdminManager {
    /// Create a new admin manager
    pub fn new() -> Self {
        Self {
            admins: HashMap::new(),
        }
    }

    /// Add a new administrator
    pub fn add_admin(&mut self, admin_id: String, public_key: PublicKey) -> Result<(), AstorError> {
        if self.admins.contains_key(&admin_id) {
            return Err(AstorError::Unauthorized("Administrator already exists".to_string()));
        }

        let admin = Administrator {
            id: admin_id.clone(),
            public_key,
            role: if self.admins.is_empty() { Role::RootAdmin } else { Role::BankAdmin },
            created_at: Utc::now(),
            is_active: true,
        };

        self.admins.insert(admin_id, admin);
        Ok(())
    }

    /// Remove an administrator
    pub fn remove_admin(&mut self, admin_id: &str, requester_id: &str) -> Result<(), AstorError> {
        let requester = self.get_admin(requester_id)?;
        
        if !requester.role.can_manage_admins() {
            return Err(AstorError::Unauthorized("Insufficient privileges to remove admin".to_string()));
        }

        if admin_id == "root" {
            return Err(AstorError::Unauthorized("Cannot remove root administrator".to_string()));
        }

        self.admins.remove(admin_id);
        Ok(())
    }

    /// Get administrator by ID
    pub fn get_admin(&self, admin_id: &str) -> Result<&Administrator, AstorError> {
        self.admins
            .get(admin_id)
            .ok_or_else(|| AstorError::AdminNotFound(admin_id.to_string()))
    }

    /// Verify admin action with signature
    pub fn verify_admin_action(
        &self,
        admin_id: &str,
        action: &[u8],
        signature: &Signature,
    ) -> Result<(), AstorError> {
        let admin = self.get_admin(admin_id)?;
        
        if !admin.is_active {
            return Err(AstorError::Unauthorized("Administrator is inactive".to_string()));
        }

        signature.verify(&admin.public_key, action)?;
        Ok(())
    }

    /// List all active administrators
    pub fn list_active_admins(&self) -> Vec<&Administrator> {
        self.admins
            .values()
            .filter(|admin| admin.is_active)
            .collect()
    }
}
