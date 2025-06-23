use std::collections::HashSet;

use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A deserialised JSON Web Token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: the user that this token represents
    pub sub: Uuid,
    /// Expiry: when the token becomes invalid
    exp: u64,
    /// Issued At
    iat: u64,
    /// Audience: the services that this token is intended for
    aud: HashSet<Service>,
    /// Issuer: the service that issued this token
    iss: Service,
    /// Actions that this token allows one to take
    pub scopes: HashSet<Scope>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Service {
    StarHavenPlatform,
    StarHavenSccache,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Can create draft mods
    CreateMod,
    /// Can publish mods that they have created
    PublishMod,
    #[serde(other)]
    Unknown,
}

impl Claims {
    pub fn new(user: Uuid, scopes: impl IntoIterator<Item = Scope>, age: u64) -> Self {
        let now = get_current_timestamp();
        Claims {
            sub: user,
            exp: now + age,
            iat: now,
            aud: HashSet::from_iter([Service::StarHavenPlatform, Service::StarHavenSccache]),
            iss: Service::StarHavenPlatform,
            scopes: HashSet::from_iter(scopes),
        }
    }

    pub fn encode(&self) -> Result<String, jsonwebtoken::errors::Error> {
        encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret("secret".as_ref()),
        ) // TODO: key
    }

    pub fn validate(token: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let mut validation = Validation::default();
        validation.set_required_spec_claims(&["sub", "exp"]);
        validation.set_audience(&["star_haven_platform"]);
        decode::<Claims>(
            token,
            &DecodingKey::from_secret("secret".as_ref()),
            &validation,
        ) // TODO: key
        .map(|token_data| token_data.claims)
    }
}
