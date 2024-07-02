use crate::{error::AppError, User};
use jwt_simple::prelude::*;
use std::ops::Deref;

const JWT_ISS: &str = "chat-server";
const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_AUD: &str = "chat-server";

pub struct EncodingKey(Ed25519KeyPair);

pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn encode(&self, user: impl Into<User>) -> Result<String, AppError> {
        let user = user.into();
        let mut claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
        claims = claims.with_issuer(JWT_ISS).with_audience(JWT_AUD);
        Ok(self.sign(claims)?)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    pub fn verify(&self, token: &str) -> Result<User, AppError> {
        // let mut options = VerificationOptions::default();

        let options = VerificationOptions {
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUD])),
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISS])),
            ..Default::default()
        };

        let claims = self.verify_token(token, Some(options))?;
        Ok(claims.custom)
    }
}

impl Deref for EncodingKey {
    type Target = Ed25519KeyPair;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for DecodingKey {
    type Target = Ed25519PublicKey;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_jwt_should_work() -> Result<()> {
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");

        let ek = EncodingKey::load(encoding_pem)?;
        let dk = DecodingKey::load(decoding_pem)?;

        let user = User::new(1, "firsteor", "firsteor@test.com");

        let token = ek.encode(user.clone())?;
        let user2 = dk.verify(&token)?;

        assert_eq!(user, user2);
        Ok(())
    }
}
