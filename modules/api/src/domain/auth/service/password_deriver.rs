use std::num::NonZeroU32;

use ring::{
    digest, pbkdf2,
    rand::{self, SecureRandom},
};

use crate::shared::AppError;

pub struct PasswordDeriver;

impl PasswordDeriver {
    pub fn compute_salt() -> Result<Vec<u8>, AppError> {
        let mut salt = [0u8; CREDENTIAL_LEN];
        rng.fill(&mut salt)
            .map_err(|_| AppError::InternalServerError)?;
    }

    pub fn derive(password: &str) -> Result<PasswordDerived, AppError> {
        const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
        let rng = rand::SystemRandom::new();

        let mut hash = [0u8; CREDENTIAL_LEN];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(10_000).unwrap(),
            &salt,
            password.as_bytes(),
            &mut hash,
        );

        Ok(PasswordDerived {
            salt: salt.to_vec(),
            hash: hash.to_vec(),
        })
    }
}

pub struct PasswordDerived {
    pub salt: Vec<u8>,
    pub hash: Vec<u8>,
}
