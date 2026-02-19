use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub network_isolation: bool,
    pub memory_limit_mb: Option<u32>,
    pub cpu_limit: Option<f32>,
    pub timeout_seconds: u32,
    pub read_only_rootfs: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            network_isolation: true,
            memory_limit_mb: Some(512),
            cpu_limit: Some(1.0),
            timeout_seconds: 30,
            read_only_rootfs: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SandboxViolationType {
    PrivilegeEscalation,
    SymlinkEscape,
    NetworkAccess,
    FileSystemAccess,
    ProcessAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxViolation {
    pub violation_type: SandboxViolationType,
    pub timestamp: DateTime<Utc>,
    pub details: String,
}

pub struct SandboxMonitor {
    violations: VecDeque<SandboxViolation>,
    max_violations: usize,
}

impl SandboxMonitor {
    pub fn new(max_violations: usize) -> Self {
        Self {
            violations: VecDeque::new(),
            max_violations,
        }
    }

    pub fn record_violation(&mut self, violation: SandboxViolation) {
        if self.violations.len() >= self.max_violations {
            self.violations.pop_front();
        }
        self.violations.push_back(violation);
    }

    pub fn detect_privilege_escalation(&mut self, command: &str) -> bool {
        if command.contains("sudo") || command.contains("su ") {
            self.record_violation(SandboxViolation {
                violation_type: SandboxViolationType::PrivilegeEscalation,
                timestamp: Utc::now(),
                details: format!("Privilege escalation attempt: {}", command),
            });
            return true;
        }
        false
    }

    pub fn detect_symlink_escape(&mut self, path: &str) -> bool {
        if path.contains("..") {
            self.record_violation(SandboxViolation {
                violation_type: SandboxViolationType::SymlinkEscape,
                timestamp: Utc::now(),
                details: format!("Symlink escape attempt: {}", path),
            });
            return true;
        }
        false
    }

    pub fn get_violations(&self) -> Vec<SandboxViolation> {
        self.violations.iter().cloned().collect()
    }

    pub fn clear_violations(&mut self) {
        self.violations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_defaults() {
        let config = SandboxConfig::default();
        assert!(config.enabled);
        assert!(config.network_isolation);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_sandbox_monitor_creation() {
        let monitor = SandboxMonitor::new(100);
        assert_eq!(monitor.get_violations().len(), 0);
    }

    #[test]
    fn test_privilege_escalation_detection() {
        let mut monitor = SandboxMonitor::new(100);
        assert!(monitor.detect_privilege_escalation("sudo rm -rf /"));
        assert_eq!(monitor.get_violations().len(), 1);
    }

    #[test]
    fn test_symlink_escape_detection() {
        let mut monitor = SandboxMonitor::new(100);
        assert!(monitor.detect_symlink_escape("../../../etc/passwd"));
        assert_eq!(monitor.get_violations().len(), 1);
    }

    #[test]
    fn test_violation_eviction() {
        let mut monitor = SandboxMonitor::new(2);

        for i in 0..3 {
            monitor.record_violation(SandboxViolation {
                violation_type: SandboxViolationType::FileSystemAccess,
                timestamp: Utc::now(),
                details: format!("Violation {}", i),
            });
        }

        assert_eq!(monitor.get_violations().len(), 2);
    }
}
