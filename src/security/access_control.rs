use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AccessAction {
    Read,
    Decrypt,
    Rotate,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AccessStatus {
    Allowed,
    Denied,
    RateLimited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessEvent {
    pub timestamp: DateTime<Utc>,
    pub credential_id: String,
    pub action: AccessAction,
    pub user: String,
    pub status: AccessStatus,
}

pub struct RateLimiter {
    window_size: Duration,
    max_requests: u32,
    requests: VecDeque<DateTime<Utc>>,
}

impl RateLimiter {
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            window_size: Duration::minutes(1),
            max_requests: max_requests_per_minute,
            requests: VecDeque::new(),
        }
    }

    pub fn check_limit(&mut self) -> Result<(), String> {
        let now = Utc::now();
        let cutoff = now - self.window_size;

        // Remove old requests outside window
        while let Some(&oldest) = self.requests.front() {
            if oldest < cutoff {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        if self.requests.len() >= self.max_requests as usize {
            return Err("Rate limit exceeded".to_string());
        }

        self.requests.push_back(now);
        Ok(())
    }

    pub fn get_current_count(&self) -> u32 {
        self.requests.len() as u32
    }
}

pub struct AccessPolicy {
    credential_id: String,
    allowed_tools: Vec<String>,
    allowed_users: Vec<String>,
    rate_limiter: RateLimiter,
    access_log: Vec<AccessEvent>,
}

impl AccessPolicy {
    pub fn new(credential_id: String, max_accesses_per_minute: u32) -> Self {
        Self {
            credential_id,
            allowed_tools: Vec::new(),
            allowed_users: Vec::new(),
            rate_limiter: RateLimiter::new(max_accesses_per_minute),
            access_log: Vec::new(),
        }
    }

    pub fn allow_tool(&mut self, tool: String) {
        self.allowed_tools.push(tool);
    }

    pub fn allow_user(&mut self, user: String) {
        self.allowed_users.push(user);
    }

    pub fn check_access(
        &mut self,
        user: &str,
        tool: &str,
        action: AccessAction,
    ) -> Result<(), String> {
        // Check user allowed
        if !self.allowed_users.is_empty() && !self.allowed_users.contains(&user.to_string()) {
            let event = AccessEvent {
                timestamp: Utc::now(),
                credential_id: self.credential_id.clone(),
                action,
                user: user.to_string(),
                status: AccessStatus::Denied,
            };
            self.access_log.push(event);
            return Err("User not allowed".to_string());
        }

        // Check tool allowed
        if !self.allowed_tools.is_empty() && !self.allowed_tools.contains(&tool.to_string()) {
            let event = AccessEvent {
                timestamp: Utc::now(),
                credential_id: self.credential_id.clone(),
                action,
                user: user.to_string(),
                status: AccessStatus::Denied,
            };
            self.access_log.push(event);
            return Err("Tool not allowed".to_string());
        }

        // Check rate limit
        match self.rate_limiter.check_limit() {
            Ok(_) => {
                let event = AccessEvent {
                    timestamp: Utc::now(),
                    credential_id: self.credential_id.clone(),
                    action,
                    user: user.to_string(),
                    status: AccessStatus::Allowed,
                };
                self.access_log.push(event);
                Ok(())
            }
            Err(_) => {
                let event = AccessEvent {
                    timestamp: Utc::now(),
                    credential_id: self.credential_id.clone(),
                    action,
                    user: user.to_string(),
                    status: AccessStatus::RateLimited,
                };
                self.access_log.push(event);
                Err("Rate limit exceeded".to_string())
            }
        }
    }

    pub fn get_access_log(&self) -> &[AccessEvent] {
        &self.access_log
    }
}

pub struct AccessControlManager {
    policies: HashMap<String, AccessPolicy>,
}

impl AccessControlManager {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    pub fn create_policy(&mut self, credential_id: String, max_accesses_per_minute: u32) {
        let policy = AccessPolicy::new(credential_id.clone(), max_accesses_per_minute);
        self.policies.insert(credential_id, policy);
    }

    pub fn get_policy_mut(&mut self, credential_id: &str) -> Option<&mut AccessPolicy> {
        self.policies.get_mut(credential_id)
    }

    pub fn check_access(
        &mut self,
        credential_id: &str,
        user: &str,
        tool: &str,
        action: AccessAction,
    ) -> Result<(), String> {
        let policy = self
            .policies
            .get_mut(credential_id)
            .ok_or("Policy not found")?;

        policy.check_access(user, tool, action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3);

        assert!(limiter.check_limit().is_ok());
        assert!(limiter.check_limit().is_ok());
        assert!(limiter.check_limit().is_ok());
        assert!(limiter.check_limit().is_err());
    }

    #[test]
    fn test_access_policy_user_check() {
        let mut policy = AccessPolicy::new("cred1".to_string(), 100);
        policy.allow_user("user1".to_string());

        assert!(policy
            .check_access("user1", "tool1", AccessAction::Read)
            .is_ok());
        assert!(policy
            .check_access("user2", "tool1", AccessAction::Read)
            .is_err());
    }

    #[test]
    fn test_access_policy_tool_check() {
        let mut policy = AccessPolicy::new("cred1".to_string(), 100);
        policy.allow_tool("shell".to_string());

        assert!(policy
            .check_access("user1", "shell", AccessAction::Read)
            .is_ok());
        assert!(policy
            .check_access("user1", "http", AccessAction::Read)
            .is_err());
    }

    #[test]
    fn test_access_policy_rate_limit() {
        let mut policy = AccessPolicy::new("cred1".to_string(), 2);

        assert!(policy
            .check_access("user1", "tool1", AccessAction::Read)
            .is_ok());
        assert!(policy
            .check_access("user1", "tool1", AccessAction::Read)
            .is_ok());
        assert!(policy
            .check_access("user1", "tool1", AccessAction::Read)
            .is_err());
    }

    #[test]
    fn test_access_control_manager() {
        let mut manager = AccessControlManager::new();
        manager.create_policy("cred1".to_string(), 100);

        assert!(manager
            .check_access("cred1", "user1", "tool1", AccessAction::Read)
            .is_ok());
        assert!(manager
            .check_access("cred2", "user1", "tool1", AccessAction::Read)
            .is_err());
    }
}
