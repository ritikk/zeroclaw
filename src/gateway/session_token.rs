use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
    pub token_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
}

pub struct TokenGenerator;

impl TokenGenerator {
    pub fn generate(user_id: String, expiration_hours: i64) -> SessionToken {
        let mut rng = rand::rng();
        let token_bytes: [u8; 32] = rng.gen();

        let token_id = general_purpose::URL_SAFE_NO_PAD.encode(&token_bytes);
        let now = Utc::now();

        SessionToken {
            token_id,
            user_id,
            created_at: now,
            expires_at: now + Duration::hours(expiration_hours),
            last_used: now,
        }
    }
}

pub struct TokenStore {
    tokens: HashMap<String, SessionToken>,
    revoked_tokens: std::collections::HashSet<String>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            revoked_tokens: std::collections::HashSet::new(),
        }
    }

    pub fn store(&mut self, token: SessionToken) {
        self.tokens.insert(token.token_id.clone(), token);
    }

    pub fn validate(&mut self, token_id: &str) -> Result<SessionToken, String> {
        if self.revoked_tokens.contains(token_id) {
            return Err("Token revoked".to_string());
        }

        let token = self
            .tokens
            .get_mut(token_id)
            .ok_or("Token not found")?
            .clone();

        if token.expires_at < Utc::now() {
            return Err("Token expired".to_string());
        }

        // Update last_used
        if let Some(t) = self.tokens.get_mut(token_id) {
            t.last_used = Utc::now();
        }

        Ok(token)
    }

    pub fn revoke(&mut self, token_id: &str) {
        self.revoked_tokens.insert(token_id.to_string());
        self.tokens.remove(token_id);
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let now = Utc::now();
        let before_len = self.tokens.len();

        self.tokens.retain(|_, token| token.expires_at > now);

        before_len - self.tokens.len()
    }

    pub fn get_token(&self, token_id: &str) -> Option<SessionToken> {
        self.tokens.get(token_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let token = TokenGenerator::generate("user123".to_string(), 24);

        assert_eq!(token.user_id, "user123");
        assert!(!token.token_id.is_empty());
        assert!(token.expires_at > token.created_at);
    }

    #[test]
    fn test_token_store_and_validate() {
        let mut store = TokenStore::new();
        let token = TokenGenerator::generate("user1".to_string(), 24);
        let token_id = token.token_id.clone();

        store.store(token);
        let validated = store.validate(&token_id).unwrap();

        assert_eq!(validated.user_id, "user1");
    }

    #[test]
    fn test_token_revocation() {
        let mut store = TokenStore::new();
        let token = TokenGenerator::generate("user1".to_string(), 24);
        let token_id = token.token_id.clone();

        store.store(token);
        store.revoke(&token_id);

        assert!(store.validate(&token_id).is_err());
    }

    #[test]
    fn test_cleanup_expired_tokens() {
        let mut store = TokenStore::new();

        // Create token that expires immediately
        let mut expired_token = TokenGenerator::generate("user1".to_string(), 0);
        expired_token.expires_at = Utc::now() - Duration::seconds(1);

        let valid_token = TokenGenerator::generate("user2".to_string(), 24);

        store.store(expired_token);
        store.store(valid_token);

        assert_eq!(store.tokens.len(), 2);

        let cleaned = store.cleanup_expired();
        assert_eq!(cleaned, 1);
        assert_eq!(store.tokens.len(), 1);
    }

    #[test]
    fn test_token_expiration() {
        let mut store = TokenStore::new();
        let mut token = TokenGenerator::generate("user1".to_string(), 0);
        token.expires_at = Utc::now() - Duration::seconds(1);

        let token_id = token.token_id.clone();
        store.store(token);

        assert!(store.validate(&token_id).is_err());
    }
}
