//! Feature flag management system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{interval, Duration};

use crate::errors::AstorError;

/// Feature flag manager
pub struct FeatureFlagManager {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
    provider: Box<dyn FeatureFlagProvider + Send + Sync>,
    refresh_interval: Duration,
}

/// Individual feature flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub key: String,
    pub enabled: bool,
    pub rollout_percentage: f64,
    pub conditions: Vec<FeatureFlagCondition>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Conditions for feature flag evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureFlagCondition {
    UserRole { roles: Vec<String> },
    Environment { environments: Vec<String> },
    UserAttribute { key: String, values: Vec<String> },
    TimeWindow { start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc> },
    Custom { rule: String },
}

/// Context for feature flag evaluation
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub user_id: Option<String>,
    pub user_role: Option<String>,
    pub environment: String,
    pub attributes: HashMap<String, String>,
}

/// Feature flag provider trait
#[async_trait::async_trait]
pub trait FeatureFlagProvider {
    async fn get_flags(&self) -> Result<HashMap<String, FeatureFlag>, AstorError>;
    async fn get_flag(&self, key: &str) -> Result<Option<FeatureFlag>, AstorError>;
}

/// Local file-based feature flag provider
pub struct LocalProvider {
    flags: HashMap<String, FeatureFlag>,
}

impl LocalProvider {
    pub fn new(flags: HashMap<String, bool>) -> Self {
        let feature_flags = flags
            .into_iter()
            .map(|(key, enabled)| {
                let flag = FeatureFlag {
                    key: key.clone(),
                    enabled,
                    rollout_percentage: if enabled { 100.0 } else { 0.0 },
                    conditions: vec![],
                    metadata: HashMap::new(),
                    updated_at: chrono::Utc::now(),
                };
                (key, flag)
            })
            .collect();

        Self {
            flags: feature_flags,
        }
    }
}

#[async_trait::async_trait]
impl FeatureFlagProvider for LocalProvider {
    async fn get_flags(&self) -> Result<HashMap<String, FeatureFlag>, AstorError> {
        Ok(self.flags.clone())
    }

    async fn get_flag(&self, key: &str) -> Result<Option<FeatureFlag>, AstorError> {
        Ok(self.flags.get(key).cloned())
    }
}

/// Remote HTTP-based feature flag provider
pub struct RemoteProvider {
    endpoint: String,
    api_key: String,
    client: reqwest::Client,
}

impl RemoteProvider {
    pub fn new(endpoint: String, api_key: String) -> Self {
        Self {
            endpoint,
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl FeatureFlagProvider for RemoteProvider {
    async fn get_flags(&self) -> Result<HashMap<String, FeatureFlag>, AstorError> {
        let response = self
            .client
            .get(&format!("{}/flags", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| AstorError::ConfigurationError(format!("Failed to fetch flags: {}", e)))?;

        let flags: HashMap<String, FeatureFlag> = response
            .json()
            .await
            .map_err(|e| AstorError::ConfigurationError(format!("Failed to parse flags: {}", e)))?;

        Ok(flags)
    }

    async fn get_flag(&self, key: &str) -> Result<Option<FeatureFlag>, AstorError> {
        let response = self
            .client
            .get(&format!("{}/flags/{}", self.endpoint, key))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| AstorError::ConfigurationError(format!("Failed to fetch flag: {}", e)))?;

        if response.status() == 404 {
            return Ok(None);
        }

        let flag: FeatureFlag = response
            .json()
            .await
            .map_err(|e| AstorError::ConfigurationError(format!("Failed to parse flag: {}", e)))?;

        Ok(Some(flag))
    }
}

impl FeatureFlagManager {
    pub fn new(
        provider: Box<dyn FeatureFlagProvider + Send + Sync>,
        refresh_interval: Duration,
    ) -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
            provider,
            refresh_interval,
        }
    }

    /// Start background refresh task
    pub async fn start_refresh_task(&self) -> Result<(), AstorError> {
        let flags = self.flags.clone();
        let provider = &self.provider;
        let mut interval = interval(self.refresh_interval);

        // Initial load
        let initial_flags = provider.get_flags().await?;
        {
            let mut flags_guard = flags.write().unwrap();
            *flags_guard = initial_flags;
        }

        // Background refresh
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                if let Ok(updated_flags) = provider.get_flags().await {
                    if let Ok(mut flags_guard) = flags.write() {
                        *flags_guard = updated_flags;
                        tracing::debug!("Feature flags refreshed");
                    }
                } else {
                    tracing::warn!("Failed to refresh feature flags");
                }
            }
        });

        Ok(())
    }

    /// Check if feature is enabled for given context
    pub fn is_enabled(&self, key: &str, context: &EvaluationContext) -> bool {
        let flags = self.flags.read().unwrap();
        
        if let Some(flag) = flags.get(key) {
            self.evaluate_flag(flag, context)
        } else {
            false
        }
    }

    /// Get feature flag value with default
    pub fn get_flag_value<T>(&self, key: &str, default: T, context: &EvaluationContext) -> T
    where
        T: Clone + for<'de> Deserialize<'de>,
    {
        let flags = self.flags.read().unwrap();
        
        if let Some(flag) = flags.get(key) {
            if self.evaluate_flag(flag, context) {
                if let Some(value) = flag.metadata.get("value") {
                    if let Ok(parsed) = serde_json::from_value(value.clone()) {
                        return parsed;
                    }
                }
            }
        }
        
        default
    }

    /// Evaluate feature flag against context
    fn evaluate_flag(&self, flag: &FeatureFlag, context: &EvaluationContext) -> bool {
        if !flag.enabled {
            return false;
        }

        // Check rollout percentage
        if flag.rollout_percentage < 100.0 {
            let hash = self.hash_context(context, &flag.key);
            let percentage = (hash % 100) as f64;
            if percentage >= flag.rollout_percentage {
                return false;
            }
        }

        // Evaluate conditions
        for condition in &flag.conditions {
            if !self.evaluate_condition(condition, context) {
                return false;
            }
        }

        true
    }

    /// Evaluate individual condition
    fn evaluate_condition(&self, condition: &FeatureFlagCondition, context: &EvaluationContext) -> bool {
        match condition {
            FeatureFlagCondition::UserRole { roles } => {
                context.user_role.as_ref().map_or(false, |role| roles.contains(role))
            }
            FeatureFlagCondition::Environment { environments } => {
                environments.contains(&context.environment)
            }
            FeatureFlagCondition::UserAttribute { key, values } => {
                context.attributes.get(key).map_or(false, |value| values.contains(value))
            }
            FeatureFlagCondition::TimeWindow { start, end } => {
                let now = chrono::Utc::now();
                now >= *start && now <= *end
            }
            FeatureFlagCondition::Custom { rule: _ } => {
                // Custom rule evaluation would be implemented here
                true
            }
        }
    }

    /// Hash context for consistent rollout
    fn hash_context(&self, context: &EvaluationContext, flag_key: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        context.user_id.hash(&mut hasher);
        flag_key.hash(&mut hasher);
        hasher.finish() as u32
    }

    /// Get all flags (for debugging/admin)
    pub fn get_all_flags(&self) -> HashMap<String, FeatureFlag> {
        self.flags.read().unwrap().clone()
    }
}
