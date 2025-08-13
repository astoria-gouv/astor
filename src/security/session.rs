//! Session management for secure user sessions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};

use crate::errors::AstorError;
use crate::security::auth::Role;

/// Session data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub mfa_verified: bool,
}

impl Session {
    /// Create a new session
    pub fn new(
        user_id: Uuid,
        role: Role,
        ip_address: String,
        user_agent: Option<String>,
        timeout_minutes: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            role,
            created_at: now,
            last_accessed: now,
            expires_at: now + Duration::minutes(timeout_minutes),
            ip_address,
            user_agent,
            is_active: true,
            mfa_verified: false,
        }
    }

    /// Check if session is valid
    pub fn is_valid(&self) -> bool {
        self.is_active && Utc::now() < self.expires_at
    }

    /// Update last accessed time and extend expiration
    pub fn refresh(&mut self, timeout_minutes: i64) {
        let now = Utc::now();
        self.last_accessed = now;
        self.expires_at = now + Duration::minutes(timeout_minutes);
    }

    /// Mark session as inactive
    pub fn invalidate(&mut self) {
        self.is_active = false;
    }

    /// Mark MFA as verified for this session
    pub fn verify_mfa(&mut self) {
        self.mfa_verified = true;
    }
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid,           // Subject (user ID)
    pub session_id: Uuid,    // Session ID
    pub role: String,        // User role
    pub exp: i64,            // Expiration time
    pub iat: i64,            // Issued at
    pub nbf: i64,            // Not before
    pub iss: String,         // Issuer
    pub aud: String,         // Audience
    pub mfa_verified: bool,  // MFA verification status
}

/// Session manager for handling user sessions
pub struct SessionManager {
    sessions: HashMap<Uuid, Session>,
    jwt_secret: String,
    session_timeout: i64,
    max_sessions_per_user: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(session_timeout: i64) -> Self {
        Self {
            sessions: HashMap::new(),
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string()),
            session_timeout,
            max_sessions_per_user: 5,
        }
    }

    /// Create a new session and return JWT token
    pub fn create_session(
        &mut self,
        user_id: Uuid,
        role: Role,
        ip_address: String,
        user_agent: Option<String>,
    ) -> Result<(String, Session), AstorError> {
        // Clean up expired sessions
        self.cleanup_expired_sessions();

        // Limit sessions per user
        self.enforce_session_limit(user_id);

        // Create new session
        let session = Session::new(
            user_id,
            role.clone(),
            ip_address,
            user_agent,
            self.session_timeout,
        );

        // Generate JWT token
        let token = self.generate_jwt_token(&session)?;

        // Store session
        self.sessions.insert(session.id, session.clone());

        Ok((token, session))
    }

    /// Validate JWT token and return session
    pub fn validate_token(&mut self, token: &str) -> Result<Session, AstorError> {
        let claims = self.decode_jwt_token(token)?;
        
        let session = self.sessions
            .get_mut(&claims.session_id)
            .ok_or(AstorError::Unauthorized("Session not found".to_string()))?;

        if !session.is_valid() {
            self.sessions.remove(&claims.session_id);
            return Err(AstorError::Unauthorized("Session expired".to_string()));
        }

        // Refresh session
        session.refresh(self.session_timeout);

        Ok(session.clone())
    }

    /// Check if session is valid
    pub async fn is_valid_session(&mut self, user_id: &str) -> Result<bool, AstorError> {
        let user_uuid = Uuid::parse_str(user_id)
            .map_err(|_| AstorError::InvalidInput("Invalid user ID format".to_string()))?;

        // Clean up expired sessions first
        self.cleanup_expired_sessions();

        // Check if user has any valid sessions
        let has_valid_session = self.sessions
            .values()
            .any(|session| session.user_id == user_uuid && session.is_valid());

        Ok(has_valid_session)
    }

    /// Invalidate session
    pub fn invalidate_session(&mut self, session_id: Uuid) -> Result<(), AstorError> {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.invalidate();
        }
        self.sessions.remove(&session_id);
        Ok(())
    }

    /// Invalidate all sessions for a user
    pub fn invalidate_user_sessions(&mut self, user_id: Uuid) -> Result<(), AstorError> {
        let session_ids: Vec<Uuid> = self.sessions
            .iter()
            .filter(|(_, session)| session.user_id == user_id)
            .map(|(id, _)| *id)
            .collect();

        for session_id in session_ids {
            self.invalidate_session(session_id)?;
        }

        Ok(())
    }

    /// Get active sessions for a user
    pub fn get_user_sessions(&self, user_id: Uuid) -> Vec<Session> {
        self.sessions
            .values()
            .filter(|session| session.user_id == user_id && session.is_valid())
            .cloned()
            .collect()
    }

    /// Generate JWT token for session
    fn generate_jwt_token(&self, session: &Session) -> Result<String, AstorError> {
        let claims = JwtClaims {
            sub: session.user_id,
            session_id: session.id,
            role: format!("{:?}", session.role),
            exp: session.expires_at.timestamp(),
            iat: session.created_at.timestamp(),
            nbf: session.created_at.timestamp(),
            iss: "astor-currency".to_string(),
            aud: "astor-api".to_string(),
            mfa_verified: session.mfa_verified,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| AstorError::CryptographicError(format!("JWT encoding error: {}", e)))
    }

    /// Decode and validate JWT token
    fn decode_jwt_token(&self, token: &str) -> Result<JwtClaims, AstorError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["astor-currency"]);
        validation.set_audience(&["astor-api"]);

        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )
        .map(|token_data| token_data.claims)
        .map_err(|e| AstorError::Unauthorized(format!("JWT validation error: {}", e)))
    }

    /// Clean up expired sessions
    fn cleanup_expired_sessions(&mut self) {
        let expired_sessions: Vec<Uuid> = self.sessions
            .iter()
            .filter(|(_, session)| !session.is_valid())
            .map(|(id, _)| *id)
            .collect();

        for session_id in expired_sessions {
            self.sessions.remove(&session_id);
        }
    }

    /// Enforce maximum sessions per user
    fn enforce_session_limit(&mut self, user_id: Uuid) {
        let mut user_sessions: Vec<(Uuid, DateTime<Utc>)> = self.sessions
            .iter()
            .filter(|(_, session)| session.user_id == user_id && session.is_valid())
            .map(|(id, session)| (*id, session.created_at))
            .collect();

        if user_sessions.len() >= self.max_sessions_per_user {
            // Sort by creation time (oldest first)
            user_sessions.sort_by(|a, b| a.1.cmp(&b.1));
            
            // Remove oldest sessions
            let sessions_to_remove = user_sessions.len() - self.max_sessions_per_user + 1;
            for i in 0..sessions_to_remove {
                self.sessions.remove(&user_sessions[i].0);
            }
        }
    }

    /// Get session statistics
    pub fn get_session_stats(&self) -> SessionStats {
        let total_sessions = self.sessions.len();
        let active_sessions = self.sessions.values().filter(|s| s.is_valid()).count();
        let expired_sessions = total_sessions - active_sessions;

        SessionStats {
            total_sessions,
            active_sessions,
            expired_sessions,
        }
    }
}

/// Session statistics
#[derive(Debug, Serialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub expired_sessions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let mut manager = SessionManager::new(60);
        let user_id = Uuid::new_v4();
        let role = Role::User;
        
        let result = manager.create_session(
            user_id,
            role,
            "127.0.0.1".to_string(),
            Some("test-agent".to_string()),
        );
        
        assert!(result.is_ok());
        let (token, session) = result.unwrap();
        assert!(!token.is_empty());
        assert_eq!(session.user_id, user_id);
        assert!(session.is_valid());
    }

    #[test]
    fn test_session_validation() {
        let mut manager = SessionManager::new(60);
        let user_id = Uuid::new_v4();
        
        let (token, _) = manager.create_session(
            user_id,
            Role::User,
            "127.0.0.1".to_string(),
            None,
        ).unwrap();
        
        let validation_result = manager.validate_token(&token);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_session_expiration() {
        let mut session = Session::new(
            Uuid::new_v4(),
            Role::User,
            "127.0.0.1".to_string(),
            None,
            -1, // Expired 1 minute ago
        );
        
        assert!(!session.is_valid());
    }
}
