use chrono::{DateTime, Duration, Utc};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CertificateInfo {
    pub subject: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub is_self_signed: bool,
}

pub struct CertificateManager {
    cert_path: PathBuf,
    key_path: PathBuf,
    auto_rotate: bool,
    rotate_before_expiry_days: u32,
}

impl CertificateManager {
    pub fn new(cert_path: PathBuf, key_path: PathBuf) -> Self {
        Self {
            cert_path,
            key_path,
            auto_rotate: true,
            rotate_before_expiry_days: 30,
        }
    }

    pub fn generate_self_signed(&self) -> Result<(), String> {
        use rcgen::{CertificateParams, IsCa};
        
        let mut params = CertificateParams::new(vec!["zeroclaw-local".to_string(), "localhost".to_string(), "127.0.0.1".to_string()]);
        params.is_ca = IsCa::NoCa;
        
        let cert = rcgen::Certificate::from_params(params)
            .map_err(|e| format!("Failed to generate certificate: {}", e))?;

        let cert_pem = cert.serialize_pem()
            .map_err(|e| format!("Failed to serialize certificate: {}", e))?;

        let key_pem = cert.serialize_private_key_pem();

        // Create directories if they don't exist
        if let Some(parent) = self.cert_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create cert directory: {}", e))?;
        }

        fs::write(&self.cert_path, cert_pem)
            .map_err(|e| format!("Failed to write certificate: {}", e))?;

        fs::write(&self.key_path, key_pem)
            .map_err(|e| format!("Failed to write key: {}", e))?;

        Ok(())
    }

    pub fn ensure_certificate_exists(&self) -> Result<(), String> {
        if !self.cert_path.exists() || !self.key_path.exists() {
            self.generate_self_signed()?;
        }
        Ok(())
    }

    pub fn should_rotate(&self) -> Result<bool, String> {
        if !self.auto_rotate {
            return Ok(false);
        }

        // For now, always return false - rotation would require parsing cert
        // In production, parse cert and check expiry
        Ok(false)
    }

    pub fn rotate_certificate(&self) -> Result<(), String> {
        // Backup old certificate
        if self.cert_path.exists() {
            let backup_path = format!("{}.backup", self.cert_path.display());
            fs::copy(&self.cert_path, &backup_path)
                .map_err(|e| format!("Failed to backup certificate: {}", e))?;
        }

        // Generate new certificate
        self.generate_self_signed()?;

        Ok(())
    }

    pub fn get_cert_path(&self) -> &PathBuf {
        &self.cert_path
    }

    pub fn get_key_path(&self) -> &PathBuf {
        &self.key_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_manager_creation() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        let manager = CertificateManager::new(cert_path.clone(), key_path.clone());

        assert_eq!(manager.get_cert_path(), &cert_path);
        assert_eq!(manager.get_key_path(), &key_path);
    }
}
