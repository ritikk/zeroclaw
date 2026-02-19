use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
    pub requires_judgment: bool,
}

pub struct CommandValidator {
    whitelist: HashSet<String>,
    blacklist: HashSet<String>,
    injection_patterns: Vec<Regex>,
}

impl CommandValidator {
    pub fn new() -> Self {
        let mut blacklist = HashSet::new();
        blacklist.insert("rm".to_string());
        blacklist.insert("dd".to_string());
        blacklist.insert("mkfs".to_string());
        blacklist.insert("fdisk".to_string());
        blacklist.insert("shutdown".to_string());
        blacklist.insert("reboot".to_string());

        let injection_patterns = vec![
            Regex::new(r"[;&|`$()]").unwrap(),
            Regex::new(r">\s*/dev/").unwrap(),
            Regex::new(r"<\s*\(").unwrap(),
        ];

        Self {
            whitelist: HashSet::new(),
            blacklist,
            injection_patterns,
        }
    }

    pub fn add_whitelist(&mut self, command: String) {
        self.whitelist.insert(command);
    }

    pub fn validate(&self, command: &str) -> CommandValidationResult {
        let cmd_name = command.split_whitespace().next().unwrap_or("");

        // Check blacklist
        if self.blacklist.contains(cmd_name) {
            return CommandValidationResult {
                is_valid: false,
                reason: Some(format!("Command '{}' is blacklisted", cmd_name)),
                requires_judgment: false,
            };
        }

        // Check injection patterns
        for pattern in &self.injection_patterns {
            if pattern.is_match(command) {
                return CommandValidationResult {
                    is_valid: true,
                    reason: None,
                    requires_judgment: true,
                };
            }
        }

        // Check whitelist if not empty
        if !self.whitelist.is_empty() && !self.whitelist.contains(cmd_name) {
            return CommandValidationResult {
                is_valid: true,
                reason: None,
                requires_judgment: true,
            };
        }

        CommandValidationResult {
            is_valid: true,
            reason: None,
            requires_judgment: false,
        }
    }

    pub fn detect_symlink_escape(&self, path: &str) -> bool {
        path.contains("..") || path.contains("~")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacklist_detection() {
        let validator = CommandValidator::new();
        let result = validator.validate("rm -rf /");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_injection_detection() {
        let validator = CommandValidator::new();
        let result = validator.validate("ls; cat /etc/passwd");
        assert!(result.requires_judgment);
    }

    #[test]
    fn test_safe_command() {
        let validator = CommandValidator::new();
        let result = validator.validate("ls -la");
        assert!(result.is_valid);
        assert!(!result.requires_judgment);
    }

    #[test]
    fn test_symlink_escape_detection() {
        let validator = CommandValidator::new();
        assert!(validator.detect_symlink_escape("../../../etc/passwd"));
        assert!(!validator.detect_symlink_escape("/home/user/file"));
    }

    #[test]
    fn test_whitelist_enforcement() {
        let mut validator = CommandValidator::new();
        validator.add_whitelist("git".to_string());

        let result = validator.validate("npm install");
        assert!(result.requires_judgment);
    }
}
