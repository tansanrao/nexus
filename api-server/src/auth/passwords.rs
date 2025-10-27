use argon2::{Algorithm, Argon2, ParamsBuilder, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use rand::RngCore;

use crate::auth::{AuthError, AuthResult};

const SALT_LEN: usize = 16;

#[derive(Clone)]
pub struct PasswordService {
    argon2: Argon2<'static>,
}

impl PasswordService {
    pub fn new() -> AuthResult<Self> {
        let mut params = ParamsBuilder::new();
        params.m_cost(19 * 1024)?; // 19 MiB
        params.t_cost(2)?;
        params.p_cost(1)?;
        let params = params.build()?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        Ok(Self { argon2 })
    }

    pub fn hash_password(&self, password: &str) -> AuthResult<String> {
        let mut salt = [0u8; SALT_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(AuthError::from)?
            .to_string();
        Ok(hash)
    }

    pub fn verify_password(&self, password: &str, encoded: &str) -> AuthResult<bool> {
        let parsed = PasswordHash::new(encoded)?;
        Ok(self
            .argon2
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_and_verifies_passwords() {
        let service = PasswordService::new().expect("password service");
        let hash = service
            .hash_password("super-secret")
            .expect("hash generation");
        assert!(service
            .verify_password("super-secret", &hash)
            .expect("verify succeeds"));
        assert!(!service
            .verify_password("wrong-password", &hash)
            .expect("verify runs"));
    }
}
