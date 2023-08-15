use std::num::NonZeroU32;

use anyhow::anyhow;
use ring::{
    digest, pbkdf2,
    rand::{self, SecureRandom},
};

#[derive(Clone)]
pub struct Kdf {
    algorithm: KdfAlgorithm,
    iterations: u32,
}

#[derive(Clone)]
pub enum KdfAlgorithm {
    Pbkdf2HmacSha256,
}

impl Kdf {
    pub fn new(algorithm: KdfAlgorithm, iterations: u32) -> Self {
        Self { algorithm, iterations }
    }

    pub fn gen_salt(&self) -> anyhow::Result<Vec<u8>> {
        let mut salt = vec![0; self.get_credential_len()];

        let rng = rand::SystemRandom::new();
        rng.fill(&mut salt).map_err(|_| anyhow!("CryptoError"))?;

        Ok(salt)
    }

    pub fn derive(&self, secret: &str, salt: &[u8]) -> anyhow::Result<Vec<u8>> {
        match self.algorithm {
            KdfAlgorithm::Pbkdf2HmacSha256 => {
                let mut hash = vec![0; self.get_credential_len()];
                pbkdf2::derive(
                    pbkdf2::PBKDF2_HMAC_SHA256,
                    NonZeroU32::new(self.iterations).unwrap(),
                    salt,
                    secret.as_bytes(),
                    &mut hash,
                );
                Ok(hash)
            }
        }
    }

    pub fn verify(&self, secret: &str, salt: &[u8], derived_key: &[u8]) -> anyhow::Result<bool> {
        match self.algorithm {
            KdfAlgorithm::Pbkdf2HmacSha256 => {
                let result = pbkdf2::verify(
                    pbkdf2::PBKDF2_HMAC_SHA256,
                    NonZeroU32::new(self.iterations).unwrap(),
                    salt,
                    secret.as_bytes(),
                    derived_key,
                );
                Ok(result.is_ok())
            }
        }
    }

    fn get_credential_len(&self) -> usize {
        match self.algorithm {
            KdfAlgorithm::Pbkdf2HmacSha256 => digest::SHA256_OUTPUT_LEN,
        }
    }
}

#[cfg(feature = "stable-test")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test() {
        let kdf = Kdf::new(KdfAlgorithm::Pbkdf2HmacSha256, 100);

        let salt = kdf.gen_salt().unwrap();
        let hash = kdf.derive("test", &salt).unwrap();

        let result_ok = kdf.verify("test", &salt, &hash).unwrap();
        assert!(result_ok);

        let result_failed = kdf.verify("test_error", &salt, &hash).unwrap();
        assert!(!result_failed);
    }
}
