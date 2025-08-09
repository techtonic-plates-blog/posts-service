use std::ops::{Deref, DerefMut};

use jsonwebtoken::{decode, Algorithm, Validation};
use poem::Request;
use poem_openapi::{SecurityScheme, auth::Bearer};
use serde::{Deserialize, Serialize};

use crate::config::CONFIG;

/// Structured permission with action and resource
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Permission {
    pub action: String,
    pub resource: String,
}

impl Permission {
    pub fn new(action: &str, resource: &str) -> Self {
        Self {
            action: action.to_string(),
            resource: resource.to_string(),
        }
    }

    /// Parse from "action:resource" format
    pub fn from_string(permission_str: &str) -> Option<Self> {
        let parts: Vec<&str> = permission_str.split(':').collect();
        if parts.len() == 2 {
            Some(Self::new(parts[0], parts[1]))
        } else {
            None
        }
    }

    /// Convert to "action:resource" format
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.action, self.resource)
    }
}

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Claims {
    pub sub: String,
    pub company: String,
    pub exp: usize,
    pub permissions: Vec<Permission>,
}

#[derive(SecurityScheme)]
#[oai(
    ty = "bearer",
    key_in = "header",
    key_name = "Bearer",
    checker = "key_checker"
)]
#[allow(dead_code)]
pub struct BearerAuthorization(pub Claims);

impl BearerAuthorization {
    /// Check if the user has a specific permission
    pub fn has_permission(&self, action: &str, resource: &str) -> bool {
        let required_permission = Permission::new(action, resource);
        self.permissions.contains(&required_permission)
    }

    /// Check if the user has any of the specified permissions
    pub fn has_any_permission(&self, permissions: &[(String, String)]) -> bool {
        permissions.iter().any(|(action, resource)| {
            self.has_permission(action, resource)
        })
    }
}

async fn key_checker(_: &Request, token: Bearer) -> Option<Claims> {
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