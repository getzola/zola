//! # Zola Encryption Middleware
//!
//! Provides build-time encryption of HTML pages with client-side decryption in the browser.
//!
//! ## ⚠️ SECURITY NOTICE
//!
//! **This is CLIENT-SIDE encryption, NOT server-side access control.**
//!
//! The encrypted content is publicly downloadable in its encrypted form. This means:
//!
//! - ❌ **NOT suitable for**: Highly sensitive data, PII, passwords, API keys, or anything
//!   requiring strong security guarantees
//! - ✅ **Suitable for**: "Soft" protection against casual browsing, unlisted content,
//!   draft previews, or content requiring simple access control
//!
//! **Key Limitations:**
//! - Unlimited brute-force attempts possible (no rate limiting)
//! - Once password is shared, access cannot be revoked without rebuilding
//! - Weak passwords are vulnerable to dictionary attacks
//! - Encrypted content is not protected from determined attackers
//! - Requires JavaScript enabled in user's browser
//!
//! ## Cryptographic Details
//!
//! - **Encryption**: AES-256-GCM (authenticated encryption)
//! - **Key Derivation**: Argon2id (for password mode)
//! - **Random Generation**: OS-provided CSPRNG
//!
//! See `ENCRYPTION.md` for full documentation and security implications.

use std::sync::Arc;

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use argon2::{Argon2, ParamsBuilder, Version};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use config::{Config, EncryptionRule};
use errors::{Result, anyhow};
use libs::globset::{Glob, GlobSet, GlobSetBuilder};
use libs::tera::{Context, Tera};
use rand::RngCore;
use utils::templates::render_template;

use super::{ContentType, Middleware, Output, OutputData, OutputKey, OutputPackage, OutputTags};

const BLOB_VERSION_PASSWORD: u8 = 1;
const BLOB_VERSION_KEY: u8 = 2;
const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const KEY_LENGTH: usize = 32;

/// Encryption blob formats:
/// Version 1 (Password): [version:1][salt:16][nonce:12][ciphertext+tag]
/// Version 2 (RawKey/GeneratedKey): [version:1][nonce:12][ciphertext+tag]
pub struct EncryptionMiddleware {
    /// Rules for which paths to encrypt and how
    rules: Vec<CompiledRule>,
    /// Tera instance for rendering vault template
    tera: Arc<Tera>,
    /// Config for template access
    config: Arc<Config>,
}

/// A compiled encryption rule with resolved glob patterns
struct CompiledRule {
    /// Original rule from config
    rule: EncryptionRule,
    /// Compiled glob set for fast path matching
    globset: GlobSet,
    /// Resolved password or key
    auth_data: AuthData,
}

/// Authentication data (password or raw key)
enum AuthData {
    /// Password to derive key from (using Argon2)
    Password(String),
    /// Raw 256-bit key (32 bytes)
    RawKey([u8; 32]),
    /// Auto-generate a random key per file and embed it in the output
    GeneratedKey,
}

impl EncryptionMiddleware {
    pub fn new(config: Arc<Config>, tera: Arc<Tera>) -> Result<Self> {
        let mut rules = Vec::new();

        for rule in &config.encrypt {
            let compiled = Self::compile_rule(rule)?;
            rules.push(compiled);
        }

        Ok(Self { rules, tera, config })
    }

    /// Compile a rule: build globset and resolve auth data
    fn compile_rule(rule: &EncryptionRule) -> Result<CompiledRule> {
        // Build globset from path patterns
        let mut builder = GlobSetBuilder::new();
        for pattern in &rule.paths {
            let glob = Glob::new(pattern).map_err(|e| {
                anyhow!("Invalid glob pattern '{}' in encryption rule: {}", pattern, e)
            })?;
            builder.add(glob);
        }
        let globset = builder.build().map_err(|e| anyhow!("Failed to build globset: {}", e))?;

        // Resolve authentication data
        let auth_data = if let Some(env_var) = &rule.password_env {
            let full_env_name = format!("ZOLA_ENCRYPTION_PASS_{}", env_var);
            let password = std::env::var(&full_env_name).map_err(|_| {
                anyhow!("Environment variable '{}' not set for encryption", full_env_name)
            })?;
            AuthData::Password(password)
        } else if let Some(password) = &rule.password {
            AuthData::Password(password.clone())
        } else if let Some(env_var) = &rule.key_env {
            let full_env_name = format!("ZOLA_ENCRYPTION_PASS_{}", env_var);
            let key_str = std::env::var(&full_env_name).map_err(|_| {
                anyhow!("Environment variable '{}' not set for encryption", full_env_name)
            })?;

            // Parse hex or base64
            let key_bytes = if key_str.len() == 64 {
                // Hex format
                hex::decode(&key_str).map_err(|e| anyhow!("Invalid hex key: {}", e))?
            } else if key_str.len() == 44 {
                // Base64 format
                BASE64.decode(&key_str).map_err(|e| anyhow!("Invalid base64 key: {}", e))?
            } else {
                return Err(anyhow!(
                    "Key must be 64 hex characters or 44 base64 characters (256 bits)"
                ));
            };

            if key_bytes.len() != 32 {
                return Err(anyhow!("Key must be exactly 32 bytes (256 bits)"));
            }

            let mut key = [0u8; 32];
            key.copy_from_slice(&key_bytes);
            AuthData::RawKey(key)
        } else {
            // No auth method specified - use auto-generated key per file
            AuthData::GeneratedKey
        };

        Ok(CompiledRule { rule: rule.clone(), globset, auth_data })
    }

    /// Build output path from OutputKey for matching
    fn output_path(key: &OutputKey) -> String {
        if key.components.is_empty() {
            key.filename.clone()
        } else {
            format!("{}/{}", key.components.join("/"), key.filename)
        }
    }

    /// Find matching encryption rule for a path
    fn find_matching_rule(&self, path: &str) -> Option<&CompiledRule> {
        self.rules.iter().find(|rule| rule.globset.is_match(path))
    }

    /// Encrypt content with the given rule
    /// Returns (encrypted_blob, optional_key_to_write)
    fn encrypt_content(
        &self,
        content: &str,
        rule: &CompiledRule,
    ) -> Result<(String, Option<String>)> {
        // Generate random nonce (always needed)
        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce_bytes);

        // Build blob based on auth type
        let (blob, key_to_save) = match &rule.auth_data {
            AuthData::Password(password) => {
                // Generate random salt for key derivation
                let mut salt_bytes = vec![0u8; SALT_LENGTH];
                OsRng.fill_bytes(&mut salt_bytes);

                // Derive key from password
                let key = self.derive_key_argon2(password.as_bytes(), &salt_bytes, rule)?;

                // Encrypt with AES-256-GCM
                let cipher = Aes256Gcm::new(&key.into());
                let nonce = Nonce::from_slice(&nonce_bytes);
                let ciphertext = cipher
                    .encrypt(nonce, content.as_bytes())
                    .map_err(|e| anyhow!("Encryption failed: {}", e))?;

                // Build blob: [version][salt][nonce][ciphertext+tag]
                let mut blob =
                    Vec::with_capacity(1 + SALT_LENGTH + NONCE_LENGTH + ciphertext.len());
                blob.push(BLOB_VERSION_PASSWORD);
                blob.extend_from_slice(&salt_bytes);
                blob.extend_from_slice(&nonce_bytes);
                blob.extend_from_slice(&ciphertext);
                (blob, None)
            }

            AuthData::RawKey(key) => {
                // Use provided key directly
                let cipher = Aes256Gcm::new(&(*key).into());
                let nonce = Nonce::from_slice(&nonce_bytes);
                let ciphertext = cipher
                    .encrypt(nonce, content.as_bytes())
                    .map_err(|e| anyhow!("Encryption failed: {}", e))?;

                // Build blob: [version][nonce][ciphertext+tag]
                let mut blob = Vec::with_capacity(1 + NONCE_LENGTH + ciphertext.len());
                blob.push(BLOB_VERSION_KEY);
                blob.extend_from_slice(&nonce_bytes);
                blob.extend_from_slice(&ciphertext);
                (blob, None)
            }

            AuthData::GeneratedKey => {
                // Generate a random key for this specific file
                let mut key = [0u8; KEY_LENGTH];
                OsRng.fill_bytes(&mut key);

                // Encrypt with AES-256-GCM
                let cipher = Aes256Gcm::new(&key.into());
                let nonce = Nonce::from_slice(&nonce_bytes);
                let ciphertext = cipher
                    .encrypt(nonce, content.as_bytes())
                    .map_err(|e| anyhow!("Encryption failed: {}", e))?;

                // Build blob: [version][nonce][ciphertext+tag]
                let mut blob = Vec::with_capacity(1 + NONCE_LENGTH + ciphertext.len());
                blob.push(BLOB_VERSION_KEY);
                blob.extend_from_slice(&nonce_bytes);
                blob.extend_from_slice(&ciphertext);

                // Save key as base64 for .key file
                let key_base64 = BASE64.encode(key);
                (blob, Some(key_base64))
            }
        };

        // Base64 encode blob
        Ok((BASE64.encode(&blob), key_to_save))
    }

    /// Derive encryption key from password using Argon2
    fn derive_key_argon2(
        &self,
        password: &[u8],
        salt: &[u8],
        rule: &CompiledRule,
    ) -> Result<[u8; 32]> {
        // Build Argon2 parameters from rule
        let mut params_builder = ParamsBuilder::new();
        params_builder.m_cost(rule.rule.argon2_memory);
        params_builder.t_cost(rule.rule.argon2_iterations);
        params_builder.p_cost(rule.rule.argon2_parallelism);
        params_builder.output_len(KEY_LENGTH);

        let params = params_builder.build().map_err(|e| anyhow!("Invalid Argon2 params: {}", e))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        // Derive raw key bytes directly (not encoded hash string)
        let mut key = [0u8; KEY_LENGTH];
        argon2
            .hash_password_into(password, salt, &mut key)
            .map_err(|e| anyhow!("Argon2 key derivation failed: {}", e))?;

        Ok(key)
    }

    /// Render vault template with encrypted data
    fn render_vault(
        &self,
        encrypted_data: &str,
        rule: &CompiledRule,
        original_metadata: &super::ContentMetadata,
    ) -> Result<String> {
        let mut context = Context::new();

        context.insert("config", &self.config.serialize(&original_metadata.language));
        context.insert("lang", &original_metadata.language);
        context.insert("encrypted_data", encrypted_data);

        // Determine auth type and pass relevant info to template
        let uses_password = matches!(rule.auth_data, AuthData::Password(_));
        context.insert("uses_password", &uses_password);

        // Pass Argon2 parameters to template for client-side key derivation (password mode only)
        if uses_password {
            context.insert("argon2_memory", &rule.rule.argon2_memory);
            context.insert("argon2_iterations", &rule.rule.argon2_iterations);
            context.insert("argon2_parallelism", &rule.rule.argon2_parallelism);
        }

        render_template("vault.html", &self.tera, context, &self.config.theme)
    }
}

impl Middleware for EncryptionMiddleware {
    fn process(&self, package: &mut OutputPackage) -> Result<()> {
        // Collect keys to encrypt (avoid cloning full Output content)
        let mut to_encrypt: Vec<(OutputKey, &CompiledRule)> = Vec::new();

        for entry in package.outputs.iter() {
            let key = entry.key();
            let output = entry.value();

            // Only encrypt primary HTML outputs
            if !output.tags.is_primary || output.content_type != ContentType::Html {
                continue;
            }

            let path = Self::output_path(key);
            if let Some(rule) = self.find_matching_rule(&path) {
                to_encrypt.push((key.clone(), rule));
            }
        }

        // Encrypt matching outputs
        for (key, rule) in to_encrypt {
            let path = Self::output_path(&key);

            // Get original content from package (avoid clone)
            let output = package
                .outputs
                .get(&key)
                .ok_or_else(|| anyhow!("Output disappeared for path: {}", path))?;

            let content = output
                .data
                .as_text()
                .ok_or_else(|| anyhow!("Cannot encrypt non-text content at path: {}", path))?;

            // Encrypt content
            let (encrypted_blob, key_to_save) = self
                .encrypt_content(content, rule)
                .map_err(|e| anyhow!("Failed to encrypt {}: {}", path, e))?;

            // Render vault template
            let vault_html = self.render_vault(&encrypted_blob, rule, &package.source_metadata)?;

            // Replace the primary output with encrypted vault
            package.outputs.insert(
                key.clone(),
                Output {
                    data: OutputData::Text(vault_html),
                    content_type: ContentType::Html,
                    tags: OutputTags { is_primary: true, ..Default::default() },
                },
            );

            // If there's a key to save, write it to a .key file
            if let Some(key_content) = key_to_save {
                // Create a new output key for the .key file
                let mut key_file_key = key.clone();
                key_file_key.filename = format!("{}.key", key_file_key.filename);

                package.outputs.insert(
                    key_file_key,
                    Output {
                        data: OutputData::Text(key_content),
                        content_type: ContentType::Text,
                        tags: OutputTags { is_primary: false, ..Default::default() },
                    },
                );
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "encryption"
    }
}
