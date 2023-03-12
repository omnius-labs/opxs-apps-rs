use std::num::NonZeroU32;

use anyhow::anyhow;
use ring::{
    digest, pbkdf2,
    rand::{self, SecureRandom},
};

const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;

pub fn salt() -> anyhow::Result<Vec<u8>> {
    let mut salt = [0u8; CREDENTIAL_LEN];

    let rng = rand::SystemRandom::new();
    rng.fill(&mut salt).map_err(|_| anyhow!("CryptoError"))?;

    Ok(salt.to_vec())
}

pub fn derive(secret: &str, salt: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(10_000).unwrap(),
        salt,
        secret.as_bytes(),
        &mut hash,
    );

    Ok(hash.to_vec())
}

pub fn verify(secret: &str, salt: &[u8], derived_key: &[u8]) -> anyhow::Result<bool> {
    let result = pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(10_000).unwrap(),
        salt,
        secret.as_bytes(),
        derived_key,
    );

    Ok(result.is_ok())
}
