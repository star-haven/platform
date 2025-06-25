use cfg_if::cfg_if;

pub use leptos::prelude::*;
pub use leptos::task::spawn_local;
pub use thiserror::Error;
pub use anyhow::{Result, Error};
pub use uuid::Uuid;
pub use serde::{Deserialize, Serialize};

pub use entity::prelude::*;
pub use entity::users::Model as User;

pub use crate::shell::Shell;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub use sea_orm::prelude::*;
        pub use sea_orm::TransactionTrait;

        pub use crate::auth::session::session;

        pub fn db() -> sea_orm::DatabaseConnection {
            expect_context()
        }
    }
}

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        pub use wasm_bindgen::prelude::*;
        pub use wasm_bindgen::JsCast;
        pub use wasm_bindgen::UnwrapThrowExt;
        pub use wasm_bindgen_futures::JsFuture;
    }
}
