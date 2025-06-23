use uuid::Uuid;

use crate::auth::{
    cookie::{get_cookie, set_cookie},
    token::{Claims, Scope},
};

/// 30 days
pub const SESSION_LENGTH_SECONDS: u64 = 30 * 24 * 60 * 60;

#[derive(Debug)]
pub struct Session {
    claims: Option<Claims>,
}

impl Session {
    /// Does this session have permission to perform actions in the given scope?
    pub fn has_scope(&self, scope: Scope) -> bool {
        if let Some(claims) = &self.claims {
            claims.scopes.contains(&scope)
        } else {
            false
        }
    }

    /// Fetch the session user from the database. `None`` if not logged in.
    pub async fn user(&self) -> Option<()> {
        todo!();
    }

    pub fn uuid(&self) -> Option<Uuid> {
        self.claims.as_ref().map(|claims| claims.sub)
    }

    pub fn is_logged_in(&self) -> bool {
        self.claims.is_some()
    }

    // TODO: take a db User, determine uuid and scopes from that
    pub fn login(&mut self) -> Result<(), jsonwebtoken::errors::Error> {
        let claims = Claims::new(Uuid::new_v4(), [], SESSION_LENGTH_SECONDS);
        set_cookie("session", &claims.encode()?, SESSION_LENGTH_SECONDS);
        self.claims = Some(claims);
        Ok(())
    }

    pub fn logout(&mut self) {
        set_cookie("session", "", SESSION_LENGTH_SECONDS);
        self.claims = None;
    }
}

#[cfg(feature = "ssr")]
pub fn session() -> Session {
    Session {
        claims: get_cookie("session").and_then(|cookie| match Claims::validate(&cookie) {
            Ok(claims) => Some(claims),
            Err(error) => {
                log::error!("token validation failed: {error}");
                None
            }
        }),
    }
}
