use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SecurityEventType {
    CredentialAccess,
    CredentialRotation,
    KeyRotation,
    RateLimitBreach,
    UnauthorizedAccess,
    EncryptionFailure,
    DecryptionFailure,
    TokenGeneration,
    TokenRevocation,
    TokenExpiration,
    TlsHandshake,
    CertificateRotation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub severity: EventSeverity,
    pub message: String,
    pub source: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

pub struct SecurityLogger {
    log_path: PathBuf,
    enable_console: bool,
}

impl SecurityLogger {
    pub fn new(log_path: PathBuf) -> Self {
        Self {
            log_path,
            enable_console: false,
        }
    }

    pub fn enable_console(&mut self) {
        self.enable_console = true;
    }

    pub fn log_event(&self, event: &SecurityEvent) -> Result<(), String> {
        let json = serde_json::to_string(event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;

        // Write to file (append-only)
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        writeln!(file, "{}", json).map_err(|e| format!("Failed to write log: {}", e))?;

        if self.enable_console {
            eprintln!("[{}] {}: {}", event.timestamp, event.event_type as u8, event.message);
        }

        Ok(())
    }

    pub fn log_credential_access(&self, credential_id: &str, user: &str) -> Result<(), String> {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::CredentialAccess,
            severity: EventSeverity::Info,
            message: format!("Credential accessed: {}", credential_id),
            source: user.to_string(),
            details: None,
        };
        self.log_event(&event)
    }

    pub fn log_unauthorized_access(&self, reason: &str) -> Result<(), String> {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::UnauthorizedAccess,
            severity: EventSeverity::Warning,
            message: "Unauthorized access attempt".to_string(),
            source: "security".to_string(),
            details: Some(reason.to_string()),
        };
        self.log_event(&event)
    }

    pub fn log_encryption_failure(&self, reason: &str) -> Result<(), String> {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::EncryptionFailure,
            severity: EventSeverity::Error,
            message: "Encryption operation failed".to_string(),
            source: "crypto".to_string(),
            details: Some(reason.to_string()),
        };
        self.log_event(&event)
    }
}

pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate_encryption_enabled(enabled: bool) -> Result<(), String> {
        if !enabled {
            return Err("Encryption must be enabled for security".to_string());
        }
        Ok(())
    }

    pub fn validate_key_rotation_interval(days: u32) -> Result<(), String> {
        if days < 30 {
            return Err("Key rotation interval must be at least 30 days".to_string());
        }
        if days > 365 {
            return Err("Key rotation interval must not exceed 365 days".to_string());
        }
        Ok(())
    }

    pub fn validate_rate_limit(limit: u32) -> Result<(), String> {
        if limit == 0 {
            return Err("Rate limit must be greater than 0".to_string());
        }
        if limit > 10000 {
            return Err("Rate limit must not exceed 10000".to_string());
        }
        Ok(())
    }

    pub fn validate_tls_enabled(enabled: bool) -> Result<(), String> {
        if !enabled {
            return Err("TLS must be enabled for gateway".to_string());
        }
        Ok(())
    }

    pub fn validate_token_expiration(hours: i64) -> Result<(), String> {
        if hours < 1 {
            return Err("Token expiration must be at least 1 hour".to_string());
        }
        if hours > 720 {
            return Err("Token expiration must not exceed 30 days".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_security_event_creation() {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::CredentialAccess,
            severity: EventSeverity::Info,
            message: "Test event".to_string(),
            source: "test".to_string(),
            details: None,
        };

        assert_eq!(event.event_type, SecurityEventType::CredentialAccess);
        assert_eq!(event.severity, EventSeverity::Info);
    }

    #[test]
    fn test_security_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("security.log");

        let logger = SecurityLogger::new(log_path.clone());
        assert_eq!(logger.log_path, log_path);
    }

    #[test]
    fn test_config_validator_encryption() {
        assert!(ConfigValidator::validate_encryption_enabled(true).is_ok());
        assert!(ConfigValidator::validate_encryption_enabled(false).is_err());
    }

    #[test]
    fn test_config_validator_key_rotation() {
        assert!(ConfigValidator::validate_key_rotation_interval(90).is_ok());
        assert!(ConfigValidator::validate_key_rotation_interval(20).is_err());
        assert!(ConfigValidator::validate_key_rotation_interval(400).is_err());
    }

    #[test]
    fn test_config_validator_rate_limit() {
        assert!(ConfigValidator::validate_rate_limit(100).is_ok());
        assert!(ConfigValidator::validate_rate_limit(0).is_err());
        assert!(ConfigValidator::validate_rate_limit(20000).is_err());
    }

    #[test]
    fn test_config_validator_tls() {
        assert!(ConfigValidator::validate_tls_enabled(true).is_ok());
        assert!(ConfigValidator::validate_tls_enabled(false).is_err());
    }

    #[test]
    fn test_config_validator_token_expiration() {
        assert!(ConfigValidator::validate_token_expiration(24).is_ok());
        assert!(ConfigValidator::validate_token_expiration(0).is_err());
        assert!(ConfigValidator::validate_token_expiration(1000).is_err());
    }
}
