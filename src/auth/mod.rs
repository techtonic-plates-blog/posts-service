use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

use jsonwebtoken::{decode, Algorithm, Validation};
use poem::Request;
use poem_openapi::{SecurityScheme, auth::Bearer};
use crate::config::CONFIG;

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Claims {
    pub sub: String,
    pub company: String,
    pub exp: usize,
    pub permissions: Vec<String>,
}

#[derive(SecurityScheme)]
#[oai(
    ty = "bearer",
    key_in = "header",
    key_name = "Bearer",
    checker="key_checker"
)]
#[allow(dead_code)]
pub struct BearerAuthorization(pub Claims);

async fn key_checker(_req: &Request, token: Bearer) -> Option<Claims> {
 let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(CONFIG.jwt_public_key.as_bytes()).ok()?;
    let Ok(token) = decode(
        &token.token,
        &decoding_key,
        &Validation::new(Algorithm::RS256),
    ) else {
        return None;
    };
    Some(token.claims)

}


impl Deref for BearerAuthorization {
    type Target = Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BearerAuthorization {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}