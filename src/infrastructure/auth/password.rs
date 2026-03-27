use anyhow::Context;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[derive(Clone, Default)]
pub struct PasswordService;

impl PasswordService {
    pub fn hash_password(&self, password: &str) -> anyhow::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .context("failed to hash password")?
            .to_string();

        Ok(hash)
    }

    pub fn verify_password(&self, password: &str, password_hash: &str) -> anyhow::Result<bool> {
        let parsed_hash = PasswordHash::new(password_hash).context("invalid password hash")?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

