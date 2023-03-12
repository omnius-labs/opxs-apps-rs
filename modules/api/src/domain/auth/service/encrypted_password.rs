use std::num::NonZeroU32;

use anyhow::anyhow;
use ring::{
    digest, pbkdf2,
    rand::{self, SecureRandom},
};

const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;

pub struct EncryptedPassword {
    pub salt: Vec<u8>,
    pub hash: Vec<u8>,
}

impl EncryptedPassword {
    pub fn new(plaintext_password: &str) -> anyhow::Result<Self> {
        let salt = Self::gen_salt()?;
        let hash = Self::derive(plaintext_password, &salt)?;

        Ok(Self { salt, hash })
    }

    pub fn gen_salt() -> anyhow::Result<Vec<u8>> {
        let mut salt = [0u8; CREDENTIAL_LEN];

        let rng = rand::SystemRandom::new();
        rng.fill(&mut salt).map_err(|_| anyhow!("CryptoError"))?;

        Ok(salt.to_vec())
    }

    pub fn derive(plaintext_password: &str, salt: &[u8]) -> anyhow::Result<Vec<u8>> {
        let mut hash = [0u8; CREDENTIAL_LEN];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(10_000).unwrap(),
            salt,
            plaintext_password.as_bytes(),
            &mut hash,
        );

        Ok(hash.to_vec())
    }
}

#[allow(unused)]
pub struct PasswordDerived {
    pub salt: Vec<u8>,
    pub hash: Vec<u8>,
}
