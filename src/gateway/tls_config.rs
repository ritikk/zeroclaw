use rustls::ServerConfig;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub struct TlsConfiguration {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub auto_generate: bool,
}

impl TlsConfiguration {
    pub fn new(cert_path: PathBuf, key_path: PathBuf) -> Self {
        Self {
            cert_path,
            key_path,
            auto_generate: true,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.cert_path.exists() && !self.auto_generate {
            return Err(format!("Certificate not found: {:?}", self.cert_path));
        }

        if !self.key_path.exists() && !self.auto_generate {
            return Err(format!("Key not found: {:?}", self.key_path));
        }

        Ok(())
    }

    pub fn load_config(&self) -> Result<Arc<ServerConfig>, String> {
        let cert_pem = fs::read(&self.cert_path)
            .map_err(|e| format!("Failed to read certificate: {}", e))?;

        let key_pem = fs::read(&self.key_path)
            .map_err(|e| format!("Failed to read key: {}", e))?;

        let certs: Vec<_> = rustls_pemfile::certs(&mut &cert_pem[..])
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Failed to parse certificate")?;

        let key = rustls_pemfile::private_key(&mut &key_pem[..])
            .map_err(|_| "Failed to parse private key")?
            .ok_or("No private key found")?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| format!("Failed to build TLS config: {}", e))?;

        Ok(Arc::new(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_configuration_creation() {
        let config = TlsConfiguration::new(
            PathBuf::from("/tmp/cert.pem"),
            PathBuf::from("/tmp/key.pem"),
        );

        assert_eq!(config.cert_path, PathBuf::from("/tmp/cert.pem"));
        assert_eq!(config.key_path, PathBuf::from("/tmp/key.pem"));
        assert!(config.auto_generate);
    }

    #[test]
    fn test_validation_fails_for_missing_files() {
        let config = TlsConfiguration {
            cert_path: PathBuf::from("/nonexistent/cert.pem"),
            key_path: PathBuf::from("/nonexistent/key.pem"),
            auto_generate: false,
        };

        assert!(config.validate().is_err());
    }
}
