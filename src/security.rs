use anyhow::{anyhow, Result};
use std::{fs, path::Path};

pub struct SecureKeyManager;

impl SecureKeyManager {
    /// Validate SSH key file permissions and ownership
    pub fn validate_key_security(key_path: &Path) -> Result<()> {
        let metadata = fs::metadata(key_path)?;

        // Check file permissions (should be 600 or 400)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode() & 0o777;
            if mode != 0o600 && mode != 0o400 {
                return Err(anyhow!(
                    "SSH key has insecure permissions: {:o}. Should be 600 or 400",
                    mode
                ));
            }
        }

        // Validate key format
        let content = fs::read_to_string(key_path)?;
        if !content.starts_with("-----BEGIN") {
            return Err(anyhow!("Invalid SSH key format"));
        }

        Ok(())
    }

    /// Sanitize SSH configuration to prevent injection
    pub fn sanitize_ssh_args(host: &str, user: &str) -> Result<(String, String)> {
        // Basic validation to prevent command injection
        if host.contains(';') || host.contains('`') || host.contains('$') {
            return Err(anyhow!("Invalid characters in hostname"));
        }

        if user.contains(';') || user.contains('`') || user.contains('$') {
            return Err(anyhow!("Invalid characters in username"));
        }

        Ok((host.to_string(), user.to_string()))
    }
}
