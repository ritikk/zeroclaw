use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAuditEntry {
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub user: String,
    pub exit_code: Option<i32>,
    pub output_truncated: bool,
    pub sandbox_used: bool,
}

pub struct CommandAuditLog {
    log_path: PathBuf,
    entries: Vec<CommandAuditEntry>,
}

impl CommandAuditLog {
    pub fn new(log_path: PathBuf) -> Self {
        Self {
            log_path,
            entries: Vec::new(),
        }
    }

    pub fn log_command(&mut self, entry: CommandAuditEntry) -> Result<(), String> {
        let json = serde_json::to_string(&entry)
            .map_err(|e| format!("Failed to serialize entry: {}", e))?;

        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        writeln!(file, "{}", json).map_err(|e| format!("Failed to write log: {}", e))?;

        self.entries.push(entry);
        Ok(())
    }

    pub fn get_entries(&self) -> &[CommandAuditEntry] {
        &self.entries
    }

    pub fn get_entries_for_user(&self, user: &str) -> Vec<CommandAuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.user == user)
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_log_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let log = CommandAuditLog::new(log_path);

        assert_eq!(log.get_entries().len(), 0);
    }

    #[test]
    fn test_log_command_entry() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let mut log = CommandAuditLog::new(log_path);

        let entry = CommandAuditEntry {
            timestamp: Utc::now(),
            command: "ls -la".to_string(),
            user: "user1".to_string(),
            exit_code: Some(0),
            output_truncated: false,
            sandbox_used: true,
        };

        assert!(log.log_command(entry).is_ok());
        assert_eq!(log.get_entries().len(), 1);
    }

    #[test]
    fn test_get_entries_for_user() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let mut log = CommandAuditLog::new(log_path);

        for i in 0..3 {
            let entry = CommandAuditEntry {
                timestamp: Utc::now(),
                command: format!("cmd{}", i),
                user: if i == 0 { "user1".to_string() } else { "user2".to_string() },
                exit_code: Some(0),
                output_truncated: false,
                sandbox_used: true,
            };
            let _ = log.log_command(entry);
        }

        let user1_entries = log.get_entries_for_user("user1");
        assert_eq!(user1_entries.len(), 1);
    }
}
